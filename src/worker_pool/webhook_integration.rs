use hyper::Client;
use hyper_tls::HttpsConnector;
use radicle::cob::patch::PatchMut;

pub struct WebhookIntegration<'a, 'g, R> {
    /** This is the worker id that instantiated this struct. */
    worker_id: usize,

    /** This is the patch that the broker needs to trigger a webhook for. */
    patch: PatchMut<'a, 'g, R>,

    url: String,
}

impl<'a, 'g, R> WebhookIntegration<'a, 'g, R> {
    pub fn execute(mut self) -> Result<(), anyhow::Error> {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        let request = hyper::Request::builder()
            .method("POST")
            .uri(self.url)
            .body(hyper::Body::empty())
            .unwrap();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let response = client.request(request).await.unwrap();

            Ok(())
        })
    }
}