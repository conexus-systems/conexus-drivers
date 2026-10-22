//! Tipos comuns a todos os módulos de driver.

/// O que se pode afirmar sobre um componente. Cresce só com evidência.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Support {
    /// Não implementado. O motivo diz o que falta.
    Unsupported(&'static str),
    /// A lógica existe e passa nos testes contra uma árvore de arquivos falsa, mas **nunca rodou num
    /// aparelho real**. Não é suporte.
    Untested,
    /// Funciona e foi testado num aparelho, com evidência C1–C5 (ADR-010). Nenhum componente chega aqui hoje.
    Supported,
}

impl Support {
    /// `true` só quando houve verificação em aparelho.
    pub fn is_verified(&self) -> bool {
        matches!(self, Support::Supported)
    }
}
