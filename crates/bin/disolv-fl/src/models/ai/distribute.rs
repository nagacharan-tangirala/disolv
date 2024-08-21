#[derive(Debug, Clone)]
pub(crate) enum DataDistributor {
    IID(IIDDistributor),
}

#[derive(Debug, Clone)]
pub(crate) struct IIDDistributor {}
