//! Módulos de driver genéricos sobre interfaces padrão do kernel Linux.
//!
//! Leem `/sys/class/thermal` e `/sys/class/power_supply`, que qualquer kernel Android expõe,
//! sem nenhum arquivo de fabricante. A raiz é parâmetro, então os testes usam uma árvore falsa.
//! **Nada aqui rodou num aparelho real**: o estado é [`Support::Untested`].

pub mod power;
pub mod sysfs;
pub mod thermal;

pub use conexus_driver_common::Support;

use std::path::PathBuf;

use conexus_core::drivers::{DriverKind, DriverManifest};

/// Componente genérico com a raiz do sysfs onde procurar.
pub trait Component {
    fn name(&self) -> &'static str;
    fn manifest(&self) -> DriverManifest;
    /// Há algo legível na raiz? Não prova que funciona num aparelho.
    fn available(&self) -> bool;
    fn support(&self) -> Support {
        Support::Untested
    }
}

fn manifest(name: &str, component: &str) -> DriverManifest {
    DriverManifest {
        name: name.into(),
        version: "0.0.1".into(),
        component: component.into(),
        kind: DriverKind::Userspace,
        kmi: None,
    }
}

pub struct Thermal {
    pub root: PathBuf,
}

impl Default for Thermal {
    fn default() -> Self {
        Thermal { root: PathBuf::from("/sys/class/thermal") }
    }
}

impl Component for Thermal {
    fn name(&self) -> &'static str {
        "generic-thermal"
    }
    fn manifest(&self) -> DriverManifest {
        manifest("generic-thermal", "thermal")
    }
    fn available(&self) -> bool {
        thermal::read_zones(&self.root).map(|z| !z.is_empty()).unwrap_or(false)
    }
}

pub struct PowerSupply {
    pub root: PathBuf,
}

impl Default for PowerSupply {
    fn default() -> Self {
        PowerSupply { root: PathBuf::from("/sys/class/power_supply") }
    }
}

impl Component for PowerSupply {
    fn name(&self) -> &'static str {
        "generic-power-supply"
    }
    fn manifest(&self) -> DriverManifest {
        manifest("generic-power-supply", "power_supply")
    }
    fn available(&self) -> bool {
        power::read_supplies(&self.root).map(|s| !s.is_empty()).unwrap_or(false)
    }
}

#[cfg(test)]
pub(crate) mod testutil {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    static N: AtomicU32 = AtomicU32::new(0);

    /// Árvore temporária que imita o sysfs e é apagada ao sair de escopo.
    pub struct TempTree(PathBuf);

    impl TempTree {
        pub fn new() -> Self {
            let p = std::env::temp_dir().join(format!("conexus-sysfs-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
            fs::create_dir_all(&p).unwrap();
            TempTree(p)
        }
        pub fn path(&self) -> &Path {
            &self.0
        }
        pub fn write(&self, rel: &str, content: &str) {
            let f = self.0.join(rel);
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, content).unwrap();
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempTree;
    use conexus_core::drivers::{DriverManager, DriverPackage, SignatureVerifier};

    struct AcceptAny;
    impl SignatureVerifier for AcceptAny {
        fn verify(&self, _m: &[u8], _s: &[u8]) -> bool {
            true
        }
    }

    #[test]
    fn manifests_pass_core_validation() {
        let mut mgr = DriverManager::new(AcceptAny);
        for c in [Box::new(Thermal::default()) as Box<dyn Component>, Box::new(PowerSupply::default())] {
            mgr.install(DriverPackage { manifest: c.manifest(), payload: vec![], signature: Some(vec![1]) }).unwrap();
        }
        assert_eq!(mgr.list().count(), 2);
    }

    #[test]
    fn support_is_never_verified_without_a_device() {
        for c in [Box::new(Thermal::default()) as Box<dyn Component>, Box::new(PowerSupply::default())] {
            assert_eq!(c.support(), Support::Untested);
            assert!(!c.support().is_verified());
        }
    }

    #[test]
    fn available_reflects_readable_tree() {
        let t = TempTree::new();
        t.write("thermal_zone0/type", "x");
        assert!(Thermal { root: t.path().into() }.available());
        let empty = TempTree::new();
        assert!(!Thermal { root: empty.path().into() }.available());
    }
}
