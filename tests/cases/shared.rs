use std::cell::RefCell;
use std::rc::Rc;
struct SharedCounterData {
    count: i64,
}
struct SharedCounter(Rc<RefCell<SharedCounterData>>);
impl SharedCounter {
    fn new() -> Self { Self(Rc::new(RefCell::new(SharedCounterData {
        count: 0,
    }))) }
    fn clone(&self) -> Self { Self(Rc::clone(&self.0)) }
    fn borrow(&self) -> std::cell::Ref<'_, SharedCounterData> { self.0.borrow() }
    fn borrow_mut(&self) -> std::cell::RefMut<'_, SharedCounterData> { self.0.borrow_mut() }
    fn inc(&self) -> () {
        self.0.borrow_mut().count = self.0.borrow_mut().count + 1;
    }
}
fn main() -> () {
    orust_runtime::install_panic_hook();
    let mut counter = SharedCounter::new();
    counter.inc();
}
