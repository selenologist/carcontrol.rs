extern crate sdl2;
extern crate byteorder;

use std::net::{ToSocketAddrs, SocketAddr, UdpSocket};
use sdl2::keyboard::Scancode;
use byteorder::{BigEndian, WriteBytesExt};

struct Connection{
    socket: UdpSocket,
    dest:   SocketAddr,
    frame:  u16,
}

impl Connection{
    fn new<A: ToSocketAddrs>(bind_addr: A, dest_addr: A) -> std::io::Result<Connection>{
        let socket = try!(UdpSocket::bind(bind_addr));
        Ok(Connection{socket: socket, dest: dest_addr.to_socket_addrs().expect("Failed to convert dest addr to SocketAddr").nth(0).unwrap(), frame: 0u16})
    }
    fn update_motors(&mut self, left: u16, right: u16){
        let current_frame = self.frame;
        
        let mut buf = [0u8; 3 * 2]; // 2 = std::mem::size_of::<u16>()
        let mut writer = std::io::Cursor::new(&mut buf[..]);
        writer.write_u16::<BigEndian>(current_frame).expect("Failed to write frame number into buffer");
        writer.write_u16::<BigEndian>(left).expect("Failed to write left motor value into buffer");
        writer.write_u16::<BigEndian>(right).expect("Failed to write right motor value into buffer");
        
        self.socket.send_to(writer.get_ref(), self.dest).expect("Failed to send UDP datagram.");

        self.frame += 1;
    }
    fn dump_dest(&self){
        println!("Destination addr: {:?}", self.dest);
    }
}

#[derive(Debug)]
enum DrivingState{
    Forward,
    ForwardLeft,
    ForwardRight,
    Left,
    Right,
    Reverse,
    ReverseLeft,
    ReverseRight,
    Stopped,
}

impl DrivingState{
    fn to_motor_values(&self) -> (u16, u16){
        use DrivingState::*;
        const FORWARD:u16 = 0;
        const MIDDLE:u16  = 90;
        const REVERSE:u16 = 180;
        match *self{
            Forward      => (FORWARD, FORWARD),
            ForwardLeft  => (MIDDLE,  FORWARD),
            ForwardRight => (FORWARD, MIDDLE ),
            Left         => (REVERSE, FORWARD),
            Right        => (FORWARD, REVERSE),
            Reverse      => (REVERSE, REVERSE),
            ReverseLeft  => (MIDDLE,  REVERSE),
            ReverseRight => (REVERSE, MIDDLE ),
            Stopped      => (MIDDLE,  MIDDLE )
        }
    }
}

struct Control{
    conn: Connection,
    state: DrivingState,
}

impl Control{
    fn key_pressed(&mut self, key_state: &sdl2::keyboard::KeyboardState){
        use DrivingState::*;
        self.state = 
            if key_state.is_scancode_pressed(Scancode::W) {
                if key_state.is_scancode_pressed(Scancode::A) {
                    ForwardLeft
                }
                else if key_state.is_scancode_pressed(Scancode::D) {
                    ForwardRight
                }
                else{
                    Forward
                }
            }
            else if key_state.is_scancode_pressed(Scancode::S) {
                if key_state.is_scancode_pressed(Scancode::A) {
                    ReverseLeft
                }
                else if key_state.is_scancode_pressed(Scancode::D) {
                    ReverseRight
                }
                else{
                    Reverse
                }
            }
            else{
                if key_state.is_scancode_pressed(Scancode::A){
                    Left
                }
                else if key_state.is_scancode_pressed(Scancode::D){
                    Right
                }
                else{
                    Stopped
                }
            };

        let (left, right) = self.state.to_motor_values();
        self.conn.update_motors(left, right);

        println!("Going {:?}", self.state);
    }
}


fn main() {
    use sdl2::event::Event;

    let conn = match Connection::new(std::env::args().nth(1).expect("First argument should be the local bind address"),
                                     std::env::args().nth(2).expect("Second argument should be the remote destination address")){
        Ok(c)  => c,
        Err(e) => { println!("Failed to bind: {:?}", e); panic!(); }
    };
    conn.dump_dest();

    let ctx = sdl2::init().unwrap();
    let video_ctx = ctx.video().unwrap();
    
    // Create a window
    let window  = match video_ctx.window("RC Car Control", 64, 64).build() {
        Ok(window) => window,
        Err(err)   => panic!("Failed to create window: {}", err)
    };

    let mut control = Control{conn: conn, state: DrivingState::Stopped};

    let mut events = ctx.event_pump().unwrap();

    // loop until we receive a QuitEvent
    let mut keys = false;

    'event : loop {
        for event in events.wait_iter() {
            match event {
                Event::Quit{..} => break 'event,
                Event::KeyDown{..} => {
                    keys = true;
                    break;
                },
                Event::KeyUp{..} =>{
                    keys = true;
                    break;
                },
                _               => continue
            }
        }
        if keys{
            let key_state = sdl2::keyboard::KeyboardState::new(&events);
            control.key_pressed(&key_state);
            keys = false;
        }
    }
}
