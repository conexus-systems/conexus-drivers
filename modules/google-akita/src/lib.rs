//! Módulo de driver do Google Pixel 8a (`akita`).
//!
//! **Estado: esqueleto.** Nenhum componente tem suporte de hardware. Cada componente declara o
//! manifest que o `conexus-driver-manager` vai verificar e responde honestamente
//! [`Support::Unsupported`]. Nenhum blob do fabricante está neste repositório.

use conexus_core::drivers::{DriverKind, DriverManifest};

pub const BRAND: &str = "google";
pub const DEVICE: &str = "akita";
/// Kernel GKI do Pixel 8a segundo `docs/architecture/DEVICE_SUPPORT.md` do conexus-mobile-os.
/// Não foi verificado em aparelho.
pub const KERNEL_RELEASE_PREFIX: &str = "6.1.";
pub const KMI: &str = "android14-6.1";

/// Resposta de um componente quando perguntado se funciona.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Support {
    /// Funciona e foi testado em aparelho, com evidência. Nenhum componente chega aqui hoje.
    Supported,
    /// Não implementado. O motivo explica o que falta.
    Unsupported(&'static str),
}

/// Componente de hardware coberto pelo módulo.
pub trait Component {
    fn name(&self) -> &'static str;
    fn manifest(&self) -> DriverManifest;
    fn support(&self) -> Support;
}

macro_rules! stub_component {
    ($ty:ident, $name:literal, $component:literal) => {
        pub struct $ty;
        impl Component for $ty {
            fn name(&self) -> &'static str {
                $name
            }
            fn manifest(&self) -> DriverManifest {
                DriverManifest {
                    name: $name.into(),
                    version: "0.0.1".into(),
                    component: $component.into(),
                    kind: DriverKind::Userspace,
                    kmi: None,
                }
            }
            fn support(&self) -> Support {
                Support::Unsupported("HAL ainda não escrito; depende de extrair e analisar o /vendor do aparelho")
            }
        }
    };
}

stub_component!(Vibrator, "akita-vibrator", "vibrator");
stub_component!(Thermal, "akita-thermal", "thermal");
stub_component!(Power, "akita-power", "power");

/// Todos os componentes do módulo.
pub fn components() -> Vec<Box<dyn Component>> {
    vec![Box::new(Vibrator), Box::new(Thermal), Box::new(Power)]
}
