/// Which field of the create-venv dialog is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateField {
    Name,
    Version,
    Packages,
    ReqFile,
    DefaultPkgs,
}

impl CreateField {
    /// Advance to the next field (wraps around).
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Version,
            Self::Version => Self::Packages,
            Self::Packages => Self::ReqFile,
            Self::ReqFile => Self::DefaultPkgs,
            Self::DefaultPkgs => Self::Name,
        }
    }

    /// Go to the previous field (wraps around).
    pub fn prev(self) -> Self {
        match self {
            Self::Name => Self::DefaultPkgs,
            Self::Version => Self::Name,
            Self::Packages => Self::Version,
            Self::ReqFile => Self::Packages,
            Self::DefaultPkgs => Self::ReqFile,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next() {
        assert_eq!(CreateField::Name.next(), CreateField::Version);
        assert_eq!(CreateField::Version.next(), CreateField::Packages);
        assert_eq!(CreateField::Packages.next(), CreateField::ReqFile);
        assert_eq!(CreateField::ReqFile.next(), CreateField::DefaultPkgs);
        assert_eq!(CreateField::DefaultPkgs.next(), CreateField::Name);
    }

    #[test]
    fn test_prev() {
        assert_eq!(CreateField::Name.prev(), CreateField::DefaultPkgs);
        assert_eq!(CreateField::Version.prev(), CreateField::Name);
        assert_eq!(CreateField::Packages.prev(), CreateField::Version);
        assert_eq!(CreateField::ReqFile.prev(), CreateField::Packages);
        assert_eq!(CreateField::DefaultPkgs.prev(), CreateField::ReqFile);
    }
}
