pub struct BDF {
    bus: u8,
    device: u8,// device&slot are the same, just different name
    function: u8
}
impl BDF{
    pub fn new(bus:u8,device:u8,function:u8)-> Self{
        Self{
            bus,
            device,
            function
        }
    }
}