// A file for preparing for the future. I hope you can get this far.

pub struct TransactionManager;

impl TransactionManager {
    pub fn new() -> Self {
        TransactionManager
    }

    pub fn begin(&self) {}

    // トランザクションのコミット
    pub fn commit(&self) {}

    // トランザクションのロールバック
    pub fn rollback(&self) {}
}
