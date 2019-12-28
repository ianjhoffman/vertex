quick_error! {
    #[derive(Debug)]
    pub enum GraphicsError {
        ContextFailed
        ShaderError
        ProgramError
        DrawError
    }
}