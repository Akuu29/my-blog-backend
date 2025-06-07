#[allow(dead_code)]
#[derive(Debug)]
pub enum ErrorLogKind {
    Authentication,
    Authorization,
    Validation,
    DomainLogic,
    Database,
    ExternalService,
    Network,
    Unexpected,
}

impl std::fmt::Display for ErrorLogKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
