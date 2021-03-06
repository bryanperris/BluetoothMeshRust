//! Network Input/Output Interface and Filter.
/*
pub trait InterfaceSink {
    fn consume_pdu(&mut self, pdu: &IncomingEncryptedNetworkPDU);
}
pub trait InputInterface<Sink: InterfaceSink> {
    fn take_sink(&mut self, sink: Sink);
}

pub struct InputInterfaces<Sink: InterfaceSink + Clone> {
    sink: Sink,
}
impl<Sink: InterfaceSink + Clone> InputInterfaces<Sink> {
    pub fn new(sink: Sink) -> Self {
        Self { sink }
    }
    pub fn add(&self, interface: &mut dyn InputInterface<Sink>) {
        interface.take_sink(self.sink.clone())
    }
}
pub trait OutputInterface {
    fn send_pdu(&mut self, pdu: &OutgoingEncryptedNetworkPDU) -> Result<(), BearerError>;
}
#[derive(Default)]
pub struct OutputInterfaces<'a> {
    interfaces: Vec<&'a mut dyn OutputInterface>,
}
impl<'a> OutputInterfaces<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_interface<'b: 'a>(&mut self, interface: &'b mut dyn OutputInterface) {
        self.interfaces.push(interface)
    }
    pub fn send_pdu(&mut self, pdu: &OutgoingEncryptedNetworkPDU) -> Result<(), BearerError> {
        for interface in self.interfaces.iter_mut() {
            (*interface).send_pdu(pdu)?
        }
        Ok(())
    }
}
*/
