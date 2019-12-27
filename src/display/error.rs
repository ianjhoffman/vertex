quick_error! {
    #[derive(Debug)]
    pub enum GraphicsError {
        ContextFailed
        ProgramError
    }
}