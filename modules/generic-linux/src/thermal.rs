//! Zonas térmicas de `/sys/class/thermal/thermal_zoneN`.

use std::fs;
use std::io;
use std::path::Path;

use crate::sysfs::{read_attr, read_i32};

/// Uma zona térmica. A temperatura vem em milésimos de grau Celsius, como o kernel informa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone {
    pub index: u32,
    /// Conteúdo de `type` (ex.: `cpu-thermal`, `battery`). Vazio se ilegível.
    pub kind: String,
    /// `None` se o arquivo `temp` falta, é ilegível ou não é um número válido.
    pub temp_millic: Option<i32>,
}

impl Zone {
    pub fn celsius(&self) -> Option<f32> {
        self.temp_millic.map(|t| t as f32 / 1000.0)
    }
}

/// Lê todas as zonas sob `root`, em ordem numérica do índice. Ignora entradas que não são `thermal_zone<N>`.
pub fn read_zones(root: &Path) -> io::Result<Vec<Zone>> {
    let mut zones = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(index) = name.strip_prefix("thermal_zone").and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        // thermal_zone01 e thermal_zone1 seriam o mesmo índice: só o nome canônico vale.
        if name != format!("thermal_zone{index}") {
            continue;
        }
        let dir = entry.path();
        zones.push(Zone {
            index,
            kind: read_attr(&dir, "type").unwrap_or_default(),
            temp_millic: read_i32(&dir, "temp"),
        });
    }
    zones.sort_by_key(|z| z.index);
    Ok(zones)
}

/// A zona mais quente entre as que têm temperatura válida.
pub fn hottest(zones: &[Zone]) -> Option<&Zone> {
    zones.iter().filter(|z| z.temp_millic.is_some()).max_by_key(|z| z.temp_millic)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Normal,
    Warm,
    Hot,
    Critical,
}

/// Limiares em milésimos de °C. Os valores padrão são **provisórios**: cada aparelho tem os seus
/// (trip points do próprio kernel) e eles precisam ser medidos antes de qualquer decisão de energia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thresholds {
    pub warm: i32,
    pub hot: i32,
    pub critical: i32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Thresholds { warm: 40_000, hot: 45_000, critical: 55_000 }
    }
}

impl Thresholds {
    /// Recusa limiares fora de ordem (warm < hot < critical).
    pub fn new(warm: i32, hot: i32, critical: i32) -> Option<Self> {
        (warm < hot && hot < critical).then_some(Thresholds { warm, hot, critical })
    }

    pub fn classify(&self, temp_millic: i32) -> Severity {
        if temp_millic >= self.critical {
            Severity::Critical
        } else if temp_millic >= self.hot {
            Severity::Hot
        } else if temp_millic >= self.warm {
            Severity::Warm
        } else {
            Severity::Normal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempTree;

    fn zone(t: &TempTree, n: &str, kind: &str, temp: Option<&str>) {
        t.write(&format!("{n}/type"), kind);
        if let Some(v) = temp {
            t.write(&format!("{n}/temp"), v);
        }
    }

    #[test]
    fn reads_zones_in_numeric_order_and_ignores_other_entries() {
        let t = TempTree::new();
        zone(&t, "thermal_zone10", "gpu", Some("41000\n"));
        zone(&t, "thermal_zone2", "cpu-thermal", Some("38500"));
        zone(&t, "thermal_zone0", "battery", Some("30100"));
        t.write("cooling_device0/type", "fan");
        t.write("thermal_zoneX/type", "lixo");
        t.write("thermal_zone01/type", "duplicado");
        let zs = read_zones(t.path()).unwrap();
        let idx: Vec<u32> = zs.iter().map(|z| z.index).collect();
        assert_eq!(idx, vec![0, 2, 10]);
        assert_eq!(zs[1].kind, "cpu-thermal");
        assert_eq!(zs[1].celsius(), Some(38.5));
    }

    #[test]
    fn invalid_or_missing_temp_is_none_not_a_panic() {
        let t = TempTree::new();
        zone(&t, "thermal_zone0", "a", None);
        zone(&t, "thermal_zone1", "b", Some("abc"));
        zone(&t, "thermal_zone2", "c", Some("99999999999999999999"));
        zone(&t, "thermal_zone3", "d", Some(""));
        zone(&t, "thermal_zone4", "e", Some("-5000"));
        let zs = read_zones(t.path()).unwrap();
        let temps: Vec<Option<i32>> = zs.iter().map(|z| z.temp_millic).collect();
        assert_eq!(temps, vec![None, None, None, None, Some(-5000)]);
    }

    #[test]
    fn hottest_skips_zones_without_reading() {
        let t = TempTree::new();
        zone(&t, "thermal_zone0", "a", Some("30000"));
        zone(&t, "thermal_zone1", "b", Some("52000"));
        zone(&t, "thermal_zone2", "c", None);
        let zs = read_zones(t.path()).unwrap();
        assert_eq!(hottest(&zs).unwrap().index, 1);
        assert!(hottest(&[]).is_none());
    }

    #[test]
    fn classify_boundaries() {
        let th = Thresholds::default();
        assert_eq!(th.classify(39_999), Severity::Normal);
        assert_eq!(th.classify(40_000), Severity::Warm);
        assert_eq!(th.classify(45_000), Severity::Hot);
        assert_eq!(th.classify(55_000), Severity::Critical);
        assert_eq!(th.classify(i32::MIN), Severity::Normal);
        assert!(Severity::Critical > Severity::Hot);
    }

    #[test]
    fn thresholds_must_be_ordered() {
        assert!(Thresholds::new(40, 45, 55).is_some());
        assert!(Thresholds::new(45, 40, 55).is_none());
        assert!(Thresholds::new(40, 40, 55).is_none());
    }

    #[test]
    fn missing_root_is_an_error_and_component_unavailable() {
        use crate::{Component, Thermal};
        let c = Thermal { root: "/caminho/que/nao/existe".into() };
        assert!(read_zones(&c.root).is_err());
        assert!(!c.available());
    }
}
