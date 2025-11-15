pub trait Create {
    fn create(&self) -> impl std::future::Future<Output = Result<(), String>>;
}

pub trait Delete {
    fn delete<R: std::io::Read>(
        &self,
        input: R,
        confirm: bool,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>;
}

pub trait Activate {
    fn activate(&self) -> impl std::future::Future<Output = ()>;
}
