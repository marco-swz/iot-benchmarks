use opcua::server::{prelude::*, callbacks};

struct Echo {}

impl callbacks::Method for Echo {
    fn call(
        &mut self,
        _session_id: &NodeId,
        _session_manager: std::sync::Arc<
            opcua::sync::RwLock<opcua::server::session::SessionManager>,
        >,
        request: &CallMethodRequest,
    ) -> Result<CallMethodResult, StatusCode> {
        
        let binding = request.input_arguments.as_ref().unwrap();
        return Ok(CallMethodResult {
            status_code: StatusCode::Good,
            input_argument_results: None,
            input_argument_diagnostic_infos: None,
            output_arguments: Some(binding.to_vec()),
        });

    }
}

fn main() {
    let server: Server = ServerBuilder::new()
        .application_name("opcua_bench")
        .application_uri("urn:opcua_bench")
        .discovery_urls(vec!["/".into()])
        .endpoint(
            "none",
            ServerEndpoint::new_none("/", &[ANONYMOUS_USER_TOKEN_ID.into()]),
        )
        .trust_client_certs()
        .multi_threaded_executor()
        .create_sample_keypair(false)
        .discovery_server_url(None)
        .host_and_port(hostname().unwrap(), 4343)
        .server()
        .unwrap();

    let ns = {
        let address_space = server.address_space();
        let mut address_space = address_space.write();
        address_space.register_namespace("urn:opcua_bench").unwrap()
    };
    let address_space = server.address_space();
    let mut address_space = address_space.write();

    let node_id = NodeId::new(ns, "echo");

    MethodBuilder::new(&NodeId::new(ns, "opcua_req"), "opcua_req", "opcua_req")
        .component_of(node_id.clone())
        .input_args(
            &mut address_space,
            &[
                ("request", DataTypeId::ByteString).into(),
            ],
        )
        .output_args(
            &mut address_space,
            &[("repsonse", DataTypeId::ByteString).into()],
        )
        .callback(Box::new(Echo{}))
        .insert(&mut address_space);

    server.run();
}

