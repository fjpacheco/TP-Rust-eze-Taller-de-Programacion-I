use std::thread::JoinHandle;
use std::{
    io::{BufRead, BufReader, Lines, Write},
    net::{Shutdown, SocketAddr, SocketAddrV4, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::native_types::{
    redis_type::encode_netcat_input, ErrorStruct, RArray, RError, RedisType,
};

use crate::messages::redis_messages;

use super::{client_atributes::client_fields::ClientFields, notifiers::Notifiers};

pub struct ClientHandler {
    stream: TcpStream,
    pub fields: Arc<Mutex<ClientFields>>,
    in_thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    out_thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    response_snd: mpsc::Sender<Option<String>>,
    addr: SocketAddrV4,
}

impl ClientHandler {
    pub fn new(mut stream_received: TcpStream, notifiers: Notifiers) -> ClientHandler {
        let in_stream = stream_received.try_clone().unwrap();
        let out_stream = stream_received.try_clone().unwrap();
        let address = get_peer(&mut stream_received).unwrap();
        let addr = address.clone();
        let fields = ClientFields::new(address);
        let shared_fields = Arc::new(Mutex::new(fields));
        let c_shared_fields = Arc::clone(&shared_fields);

        let (response_snd, response_recv): (
            mpsc::Sender<Option<String>>,
            mpsc::Receiver<Option<String>>,
        ) = mpsc::channel();
        let response_snd_clone = response_snd.clone();
        let in_thread = thread::spawn(move || {
            read_socket(in_stream, c_shared_fields, notifiers, response_snd_clone)
        });

        let out_thread = thread::spawn(move || write_socket(out_stream, response_recv));

        ClientHandler {
            stream: stream_received,
            fields: shared_fields,
            in_thread: Some(in_thread),
            out_thread: Some(out_thread),
            response_snd,
            addr,
        }
    }

    pub fn is_subscripted_to(&self, channel: &str) -> bool {
        self.fields.lock().unwrap().is_subscripted_to(channel)
    }

    pub fn is_monitor_notificable(&self) -> bool {
        self.fields.lock().unwrap().is_monitor_notifiable()
    }

    pub fn get_peer(&mut self) -> Option<SocketAddrV4> {
        get_peer(&mut self.stream)
    }

    pub fn get_addr(&self) -> String {
        self.addr.clone().to_string()
    }

    pub fn write_stream(&self, response: String) -> Result<(), ErrorStruct> {
        send_response(response, &self.response_snd)
    }

    pub fn get_detail(&self) -> String {
        self.fields.lock().unwrap().get_detail()
    }
}

fn write_socket(
    mut stream: TcpStream,
    response_recv: mpsc::Receiver<Option<String>>,
) -> Result<(), ErrorStruct> {
    for packed_response in response_recv.iter() {
        if let Some(response) = packed_response {
            let result = stream.write_all(response.as_bytes());
            if result.is_err() {
                return Err(ErrorStruct::from(redis_messages::closed_socket()));
            }
        } else {
            return Ok(());
        }
    }

    Ok(())
}

fn read_socket(
    stream: TcpStream,
    c_shared_fields: Arc<Mutex<ClientFields>>,
    notifiers: Notifiers,
    response_snd: mpsc::Sender<Option<String>>,
) -> Result<(), ErrorStruct> {
    let buf_reader_stream = BufReader::new(stream.try_clone().unwrap());
    let mut lines_buffer_reader = buf_reader_stream.lines();
    let mut response;
    while let Some(received) = lines_buffer_reader.next() {
        match received {
            Ok(input) => {
                if input.starts_with('*') {
                    response = process_command_redis(
                        input,
                        &mut lines_buffer_reader,
                        Arc::clone(&c_shared_fields),
                        &notifiers,
                    );
                } else {
                    response =
                        process_command_string(input, Arc::clone(&c_shared_fields), &notifiers);
                }
                send_response(response, &response_snd)?;
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => break, // FOR TIMEOUT OF REDIS.CONF
                _ => {
                    response = RError::encode(ErrorStruct::new(
                        "ERR".to_string(),
                        format!("Error received in next line.\nDetail: {:?}", err),
                    ));
                    send_response(response, &response_snd)?;
                    return Err(ErrorStruct::new(
                        "ERR".to_string(),
                        format!("Error received in next line.\nDetail: {:?}", err),
                    ));
                }
            },
        }
    }
    notifiers.off_client(stream.peer_addr().unwrap().to_string());
    Ok(())
}

fn process_command_redis(
    mut input: String,
    mut lines_buffer_reader: &mut Lines<BufReader<TcpStream>>,
    client_status: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> String {
    input.remove(0);
    process_command_general(input, &mut lines_buffer_reader, client_status, notifiers)
}

fn process_command_string(
    input: String,
    client_status: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> String {
    let mut input_encoded = encode_netcat_input(input);
    input_encoded.remove(0);
    let mut lines = BufReader::new(input_encoded.as_bytes()).lines();
    let first_lecture = lines.next().unwrap().unwrap_or_else(|_| "-1".into());
    process_command_general(first_lecture, &mut lines, client_status, &notifiers)
}

fn process_command_general<G>(
    first_lecture: String,
    lines_buffer_reader: &mut Lines<G>,
    client_status: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> String
where
    G: BufRead,
{
    match RArray::decode(first_lecture, lines_buffer_reader) {
        Ok(command_vec) => {
            let allowed = client_status.lock().unwrap().is_allowed_to(&command_vec[0]);

            match allowed {
                Ok(()) => delegate_command(command_vec, client_status, notifiers),
                Err(error) => RError::encode(error),
            }
        }
        Err(error) => RError::encode(error),
    }
}

fn delegate_command(
    command_received: Vec<String>,
    client_fields: Arc<Mutex<ClientFields>>,
    notifiers: &Notifiers,
) -> String {
    let command_received_initial = command_received.clone();
    let (sender, receiver): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

    let _a =
        notifiers.send_command_delegator((command_received, sender, Arc::clone(&client_fields)));

    match receiver.recv() {
        Ok(response) => {
            notifiers.notify_successful_shipment(client_fields, command_received_initial);
            response
        }
        Err(err) => RError::encode(ErrorStruct::new(
            "ERR".to_string(),
            format!("failed to receive channel content. Detail {:?}", err),
        )),
    }
}

pub fn get_peer(stream: &mut TcpStream) -> Option<SocketAddrV4> {
    match stream.peer_addr().unwrap() {
        SocketAddr::V4(addr) => Some(addr),
        SocketAddr::V6(_) => None,
    }
}

fn send_response(
    response: String,
    sender: &mpsc::Sender<Option<String>>,
) -> Result<(), ErrorStruct> {
    if sender.send(Some(response)).is_ok() {
        Ok(())
    } else {
        Err(ErrorStruct::from(redis_messages::closed_sender()))
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        self.stream
            .shutdown(Shutdown::Both)
            .expect("Error to close TcpStream");
        if let Some(handle) = self.in_thread.take() {
            match handle.join() {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        println!("COMIENZA EL DROP");
        self.response_snd.send(None).unwrap();
        if let Some(handle) = self.out_thread.take() {
            match handle.join() {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        println!("ME ELIMINE");
    }
}
