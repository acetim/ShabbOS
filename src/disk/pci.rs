
use x86_64::instructions::port::{PortGeneric, ReadOnlyAccess, WriteOnlyAccess};
use crate::dbg;
use crate::disk::pci_bdf::BDF;

const PCI_IN: u16 = 0xCFC;
const PCI_OUT: u16 =0xCF8;
pub struct PciScanner{
    out_port:PortGeneric<u32,WriteOnlyAccess>,
    in_port:PortGeneric<u32,ReadOnlyAccess>
}

impl PciScanner{
    pub fn new()-> Self {
        Self{
            out_port:PortGeneric::new(PCI_OUT),
            in_port:PortGeneric::new(PCI_IN)
        }
    }
    fn pci_read_dword(&mut self, bus:u8, slot:u8, func:u8,offset:u8)->u32{
        /*
        this function reads config data from devices via pci
        returns the header
         */
        let address:u32;
        let bus32 = bus as u32;
        let slot32 = slot as u32;
        let func32=func as u32;
        let offset32 = offset as u32;
        address = (bus32<<16)|(slot32<<11)|(func32<<8)|(offset32&0xFC)|0x80000000u32;
        unsafe{
            self.out_port.write(address);
            self.in_port.read()
        }
    }
    fn get_vendor_id(&mut self, bus:u8, slot:u8, func:u8)->u16{
        //extracts the vendor number from the header
        (self.pci_read_dword(bus,slot,func,0x0) & (u16::MAX as u32)) as u16
    }
    fn get_class_and_subclass(&mut self, bus:u8, slot:u8, func:u8)->(u8,u8){
        let class = (self.pci_read_dword(bus, slot, func, 0x8)>>16) as u16;
        (
            (class >>8)as u8, //class
            (class & (u8::MAX as u16)) as u8 //subclass
        )
    }
    pub fn check_all_busses_for_class(&mut self, target_class:(u8,u8))->Option<BDF>{
        /*
        scans all busses and all devices for a specific class/subclass combination
        returns an Option that is None when target isnt found
         */
        for bus in 0..=255{
            for slot in 0..32{
                for func in 0..8{
                    let vendor=self.get_vendor_id(bus,slot,func);
                    if  vendor != 0xFFFF{
                        let class =self.get_class_and_subclass(bus, slot, func);
                        if(class.0==target_class.0 && class.1==target_class.1){
                            dbg!("found target on bus {} slot {} func {}",bus,slot,func);
                            return Some(BDF::new(bus,slot,func))
                        }
                    }
                }
            }
        }

        None
    }

}

