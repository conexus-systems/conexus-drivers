//! Fontes de energia de `/sys/class/power_supply/<nome>`.

use std::fs;
use std::io;
use std::path::Path;

use crate::sysfs::{read_attr, read_i32, read_u32_max};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Battery,
    /// Carregador de parede, USB ou outra fonte externa.
    External,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

/// Uma fonte de energia. Campos ausentes ou inválidos ficam `None`: nada é inventado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Supply {
    pub name: String,
    pub kind: Kind,
    pub status: Status,
    /// 0..=100. Valor fora da faixa vira `None`.
    pub capacity: Option<u8>,
    pub online: Option<bool>,
    /// Décimos de °C, como o kernel informa (305 = 30,5 °C).
    pub temp_decic: Option<i32>,
    pub voltage_uv: Option<i32>,
    /// O **sinal** de `current_now` depende do fabricante (alguns usam negativo para descarga).
    /// Este campo guarda o valor cru.
    pub current_ua: Option<i32>,
}

impl Supply {
    pub fn temp_celsius(&self) -> Option<f32> {
        self.temp_decic.map(|t| t as f32 / 10.0)
    }
}

fn parse_kind(s: &str) -> Kind {
    match s.to_ascii_lowercase().as_str() {
        "battery" => Kind::Battery,
        "mains" | "usb" | "usb_dcp" | "usb_cdp" | "usb_aca" | "usb_c" | "usb_pd" | "usb_pd_drp" | "wireless" => Kind::External,
        _ => Kind::Unknown,
    }
}

fn parse_status(s: &str) -> Status {
    match s.to_ascii_lowercase().as_str() {
        "charging" => Status::Charging,
        "discharging" => Status::Discharging,
        "full" => Status::Full,
        "not charging" => Status::NotCharging,
        _ => Status::Unknown,
    }
}

fn valid_name(n: &str) -> bool {
    !n.is_empty() && n.len() <= 64 && n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// Lê todas as fontes sob `root`, em ordem alfabética.
pub fn read_supplies(root: &Path) -> io::Result<Vec<Supply>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !valid_name(&name) {
            continue;
        }
        let dir = entry.path();
        out.push(Supply {
            kind: read_attr(&dir, "type").map(|s| parse_kind(&s)).unwrap_or(Kind::Unknown),
            status: read_attr(&dir, "status").map(|s| parse_status(&s)).unwrap_or(Status::Unknown),
            capacity: read_u32_max(&dir, "capacity", 100).map(|c| c as u8),
            online: read_u32_max(&dir, "online", 1).map(|v| v == 1),
            temp_decic: read_i32(&dir, "temp"),
            voltage_uv: read_i32(&dir, "voltage_now"),
            current_ua: read_i32(&dir, "current_now"),
            name,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// A primeira bateria encontrada.
pub fn battery(supplies: &[Supply]) -> Option<&Supply> {
    supplies.iter().find(|s| s.kind == Kind::Battery)
}

/// Há alguma fonte externa online? `None` se nenhuma informou o estado.
pub fn external_power(supplies: &[Supply]) -> Option<bool> {
    let ext: Vec<&Supply> = supplies.iter().filter(|s| s.kind == Kind::External).collect();
    if ext.iter().all(|s| s.online.is_none()) {
        return None;
    }
    Some(ext.iter().any(|s| s.online == Some(true)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempTree;

    #[test]
    fn reads_a_full_battery() {
        let t = TempTree::new();
        t.write("battery/type", "Battery\n");
        t.write("battery/status", "Charging\n");
        t.write("battery/capacity", "87\n");
        t.write("battery/temp", "305\n");
        t.write("battery/voltage_now", "4200000\n");
        t.write("battery/current_now", "-850000\n");
        let s = read_supplies(t.path()).unwrap();
        let b = battery(&s).unwrap();
        assert_eq!((b.status, b.capacity), (Status::Charging, Some(87)));
        assert_eq!(b.temp_celsius(), Some(30.5));
        assert_eq!((b.voltage_uv, b.current_ua), (Some(4_200_000), Some(-850_000)));
    }

    #[test]
    fn capacity_out_of_range_or_garbage_is_none() {
        let t = TempTree::new();
        for (n, v) in [("a", "101"), ("b", "-5"), ("c", "abc"), ("d", ""), ("e", "100"), ("f", "0")] {
            t.write(&format!("{n}/type"), "Battery");
            t.write(&format!("{n}/capacity"), v);
        }
        let caps: Vec<Option<u8>> = read_supplies(t.path()).unwrap().iter().map(|s| s.capacity).collect();
        assert_eq!(caps, vec![None, None, None, None, Some(100), Some(0)]);
    }

    #[test]
    fn unknown_text_maps_to_unknown_not_an_error() {
        let t = TempTree::new();
        t.write("x/type", "Coisa Nova");
        t.write("x/status", "Exploding");
        let s = &read_supplies(t.path()).unwrap()[0];
        assert_eq!((s.kind, s.status), (Kind::Unknown, Status::Unknown));
    }

    #[test]
    fn external_power_detection() {
        let t = TempTree::new();
        t.write("battery/type", "Battery");
        t.write("usb/type", "USB");
        t.write("usb/online", "1");
        t.write("ac/type", "Mains");
        t.write("ac/online", "0");
        assert_eq!(external_power(&read_supplies(t.path()).unwrap()), Some(true));
        t.write("usb/online", "0");
        assert_eq!(external_power(&read_supplies(t.path()).unwrap()), Some(false));
        let t2 = TempTree::new();
        t2.write("usb/type", "USB");
        assert_eq!(external_power(&read_supplies(t2.path()).unwrap()), None);
    }

    #[test]
    fn online_only_accepts_zero_or_one() {
        let t = TempTree::new();
        t.write("usb/type", "USB");
        t.write("usb/online", "2");
        assert_eq!(read_supplies(t.path()).unwrap()[0].online, None);
    }

    #[test]
    fn ignores_directories_with_dangerous_names_and_sorts() {
        let t = TempTree::new();
        t.write("zeta/type", "Battery");
        t.write("alpha/type", "USB");
        t.write("bad name/type", "Battery");
        let names: Vec<String> = read_supplies(t.path()).unwrap().into_iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["alpha", "zeta"]);
    }

    #[test]
    fn missing_root_is_an_error_and_component_unavailable() {
        use crate::{Component, PowerSupply};
        let c = PowerSupply { root: "/caminho/que/nao/existe".into() };
        assert!(read_supplies(&c.root).is_err());
        assert!(!c.available());
    }
}
