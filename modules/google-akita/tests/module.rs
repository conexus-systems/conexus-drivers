use conexus_core::drivers::{
    DeviceInfo, DriverBundle, DriverError, DriverManager, DriverPackage, DriverVariant, SignatureVerifier,
};
use conexus_driver_google_akita::{components, DEVICE, KERNEL_RELEASE_PREFIX, KMI};

/// Aceita qualquer assinatura: aqui só se testa a validade dos manifests e a seleção do bundle.
struct AcceptAny;
impl SignatureVerifier for AcceptAny {
    fn verify(&self, _message: &[u8], _signature: &[u8]) -> bool {
        true
    }
}

fn bundle() -> DriverBundle {
    DriverBundle {
        brand: "google".into(),
        variants: components()
            .iter()
            .map(|c| DriverVariant {
                device: DEVICE.into(),
                kernel_release: format!("{KERNEL_RELEASE_PREFIX}99-android14"),
                package: DriverPackage { manifest: c.manifest(), payload: vec![], signature: Some(vec![1]) },
            })
            .collect(),
    }
}

fn device(kernel: &str) -> DeviceInfo {
    DeviceInfo { device: DEVICE.into(), kernel_release: kernel.into(), kmi: KMI.into() }
}

#[test]
fn nenhum_componente_declara_suporte_sem_evidencia() {
    for c in components() {
        assert!(!c.support().is_verified(), "{} não pode declarar suporte sem evidência", c.name());
    }
}

#[test]
fn manifests_passam_na_validacao_do_core() {
    let mut mgr = DriverManager::new(AcceptAny);
    mgr.install_bundle(&bundle(), &device("6.1.99-android14")).unwrap();
    assert_eq!(mgr.list().count(), components().len());
}

#[test]
fn bundle_nao_serve_para_outro_kernel() {
    let mut mgr = DriverManager::new(AcceptAny);
    let err = mgr.install_bundle(&bundle(), &device("5.15.1-android13")).unwrap_err();
    assert!(matches!(err, DriverError::NoMatchingVariant { .. }));
}

#[test]
fn sem_verificador_configurado_nada_instala() {
    let mut mgr = DriverManager::default();
    let err = mgr.install_bundle(&bundle(), &device("6.1.99-android14")).unwrap_err();
    assert!(matches!(err, DriverError::BadSignature(_)));
}

#[test]
fn nomes_de_componentes_sao_unicos() {
    let mut names: Vec<_> = components().iter().map(|c| c.name()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), components().len());
}
