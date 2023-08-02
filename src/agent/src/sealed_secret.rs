

#[cfg(test)]
mod tests {
    use protocols::sealed_secret_ttrpc_async;
    use protocols::sealed_secret_ttrpc;
    use protocols::sealed_secret;
    use std::sync::Arc;
    use async_trait::async_trait;
    use ttrpc::error::{Error, Result};
    use ttrpc::proto::{Code, Status};
    use tokio::signal::unix::{signal, SignalKind};

    const SOCK_ADDR: &str = r"unix:///tmp/ttrpc-test";

    struct TestService;

    #[async_trait]
    impl sealed_secret_ttrpc_async::SealedSecretService for TestService {
        async fn unseal_secret(
            &self,
            _ctx: &::ttrpc::asynchronous::TtrpcContext,
            _req: sealed_secret::UnSealSecretInput,
            ) -> Result<sealed_secret::UnSealSecretOutput> {
            let mut status = Status::new();
            status.set_code(Code::NOT_FOUND);
            status.set_message("Just for fun".to_string());
            println!("service: unseal_secret");
            Err(Error::RpcStatus(status))
        }
    }

    fn remove_if_sock_exist(sock_addr: &str) -> std::io::Result<()> {
        let path = sock_addr
            .strip_prefix("unix://")
            .expect("socket address is not expected");

        if std::path::Path::new(path).exists() {
            std::fs::remove_file(path)?;
        }

        Ok(())
    }

    fn unseal_secret() {
        let c = ttrpc::Client::connect(SOCK_ADDR).unwrap();
        let sc = sealed_secret_ttrpc::SealedSecretServiceClient::new(c.clone());
        let input = sealed_secret::UnSealSecretInput::new();
        let output = sc.unseal_secret(ttrpc::context::with_timeout(20 * 1000 * 1000), &input).unwrap();
        println!("output: {}", output);
    }

    fn start_ttrpc_server() {
        tokio::spawn(async move {
            let ss = Box::new(TestService {}) as Box<dyn sealed_secret_ttrpc_async::SealedSecretService  + Send + Sync>;
            let ss = Arc::new(ss);
            let ss_service = sealed_secret_ttrpc_async::create_sealed_secret_service(ss);

            remove_if_sock_exist(SOCK_ADDR).unwrap();

            let mut server = ttrpc::asynchronous::Server::new()
                .bind(SOCK_ADDR)
                .unwrap()
                .register_service(ss_service);

            server.start().await.unwrap();

            let mut interrupt = signal(SignalKind::interrupt()).unwrap();
            tokio::select! {
                _ = interrupt.recv() => {
                    // test graceful shutdown
                    println!("graceful shutdown");
                    server.shutdown().await.unwrap();
                }
            };
        });
    }

    #[test]
    fn test_sealed_secret() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        start_ttrpc_server();
        std::thread::sleep(std::time::Duration::from_secs(2));

        unseal_secret();

        rt.shutdown_background();
    }
}
