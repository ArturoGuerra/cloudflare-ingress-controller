use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::networking::v1::Ingress;
#[allow(unused_imports)]
use kube::{
    api::{Api, ResourceExt},
    runtime::{reflector, reflector::Store, watcher, WatchStreamExt},
    Client,
};
use std::env;

const INGRESS_CONTROLLER: &str = "cloudflare.ar2ro.io/controller";

#[tokio::main]
async fn main() -> Result<(), watcher::Error> {
    let is_default_ingress_class = env::var("DEFAULT_INGRESS_CLASS").is_ok();

    let k8s_client = Client::try_default().await.unwrap();

    let ingress_api: Api<Ingress> = Api::all(k8s_client.clone());

    let (reader, writer) = reflector::store::<Ingress>();
    let wc = watcher::Config::default()
        .fields("ingressClassName=cloudflare")
        .timeout(20);
    let rt = reflector(writer, watcher(ingress_api, wc));

    tokio::spawn(async move {
        loop {
            println!("Reader: {:?}", &reader);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    rt.applied_objects()
        .try_for_each(|_| async move { Ok(()) })
        .await?;

    Ok(())
}
