use openidconnect::{
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, RedirectUrl,
    core::{CoreClient, CoreProviderMetadata},
};

/// CoreClient type after OIDC discovery: auth_url is set, token/userinfo are maybe-set.
pub type OidcClient = CoreClient<
    EndpointSet,      // auth_url
    EndpointNotSet,   // device_auth_url
    EndpointNotSet,   // introspection_url
    EndpointNotSet,   // revocation_url
    EndpointMaybeSet, // token_url
    EndpointMaybeSet, // userinfo_url
>;

pub async fn build_oidc_client(
    issuer_url: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
) -> anyhow::Result<OidcClient> {
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let issuer = IssuerUrl::new(issuer_url.to_string())?;
    let metadata = CoreProviderMetadata::discover_async(issuer, &http_client).await?;
    let client = CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?);
    Ok(client)
}
