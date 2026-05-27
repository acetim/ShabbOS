use spin::MutexGuard;

pub struct Locker<T>{
    inner: spin::Mutex<T>
}
impl<T> Locker<T>{
    pub const fn new(inner:T)->Self{
        Locker{inner:spin::Mutex::new(inner)}
    }
    pub fn lock(&self)-> MutexGuard<T>{
        self.inner.lock()
    }
}