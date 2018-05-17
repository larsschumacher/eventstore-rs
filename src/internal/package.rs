use bytes::{ Bytes, BytesMut, BufMut };
use protobuf::{ Message, MessageStatic, parse_from_carllerche_bytes };
use uuid::Uuid;

use internal::command::Cmd;
use internal::messages;
use types::Credentials;

pub struct Pkg {
    pub cmd: Cmd,
    pub creds_opt: Option<Credentials>,
    pub correlation: Uuid,
    pub payload: Bytes,
}

static CLIENT_VERSION: i32 = 1;
pub static PKG_MANDATORY_SIZE: usize = 18;

impl Pkg {
    pub fn new(cmd: Cmd, correlation: Uuid) -> Pkg {
        Pkg {
            cmd,
            correlation,
            creds_opt: None,
            payload: Default::default(),
        }
    }

    pub fn from_message<M>(cmd: Cmd, creds_opt: Option<Credentials>, msg: &M)
        -> ::std::io::Result<Pkg>
        where M: Message
    {
        let mut writer =
            BytesMut::with_capacity(msg.compute_size() as usize).writer();

        msg.write_to_writer(&mut writer)?;

        let pkg = Pkg {
            cmd,
            creds_opt,
            correlation: Uuid::new_v4(),
            payload: writer.into_inner().freeze(),
        };

        Ok(pkg)
    }

    pub fn from_bytes(cmd: Cmd, creds_opt: Option<Credentials>, payload: Bytes) -> Pkg {
        Pkg {
            cmd,
            creds_opt,
            payload,
            correlation: Uuid::new_v4(),
        }
    }

    pub fn size(&self) -> usize {
        let creds_size = {
            match self.creds_opt {
                Some(ref creds) => creds.network_size(),
                None            => 0,
            }
        };

        PKG_MANDATORY_SIZE + self.payload.len() + creds_size
    }

    pub fn heartbeat_request() -> Pkg {
        Pkg::new(Cmd::HeartbeatRequest, Uuid::new_v4())
    }

    pub fn authenticate(creds: Credentials) -> Pkg {
        Pkg {
            cmd: Cmd::Authenticate,
            correlation: Uuid::new_v4(),
            creds_opt: Some(creds),
            payload: Default::default(),
        }
    }

    pub fn identify_client(name_opt: &Option<String>) -> Pkg {
        let mut msg  = messages::IdentifyClient::new();
        let     name = match *name_opt {
            Some(ref name) => name.clone(),
            None           => format!("ES-{}", Uuid::new_v4()),
        };

        msg.set_connection_name(name);
        msg.set_version(CLIENT_VERSION);

        let mut writer =
            BytesMut::with_capacity(msg.compute_size() as usize).writer();

        msg.write_to_writer(&mut writer).unwrap();

        Pkg {
            cmd: Cmd::IdentifyClient,
            correlation: Uuid::new_v4(),
            creds_opt: None,
            payload: writer.into_inner().freeze(),
        }
    }

    // Copies the Pkg except its payload.
    pub fn copy_headers_only(&self) -> Pkg {
        Pkg {
            cmd:         self.cmd,
            correlation: self.correlation,
            payload:     Default::default(),
            creds_opt:   None,
        }
    }

    pub fn to_message<M>(&self) -> ::std::io::Result<M>
        where M: MessageStatic
    {
        parse_from_carllerche_bytes(&self.payload).map_err(|e| e.into())
    }
}
