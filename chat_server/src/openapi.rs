use axum::Router;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use chat_core::models::{
    Chat, CreateChat, CreateMessage, CreateUser, CreateWorkspace, ListMessages, Messages,
    SigninUser, User, Workspace,
};

use crate::handlers::*;
use crate::ChatState;

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        list_users_handler, list_workspace_handler, create_workspace_handler, list_messages_handler
    ),
    components(schemas(
        Chat,
        CreateChat,
        CreateMessage,
        Messages,
        ListMessages,
        CreateUser,
        SigninUser,
        User,
        Workspace,
        CreateWorkspace
    )),
    tags(
        (name = "ChatServer", description = "ChatServer API")
    )
)]
struct ApiDoc;

struct SecurityAddon;

pub trait OpenApiRouter {
    fn add_openapi(self) -> Self;
}

impl OpenApiRouter for Router<ChatState> {
    fn add_openapi(self) -> Self {
        self.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
            // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
            // via SwaggerUi instead we only make rapidoc to point to the existing doc.
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    }
}

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            )
        }
    }
}
