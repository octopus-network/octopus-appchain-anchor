pub trait AppchainLifecycleManager {
    ///
    fn go_booting(&mut self);
    ///
    fn go_live(&mut self);
}
