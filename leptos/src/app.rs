use crate::router::AppRouter;
use crate::context::{provide_company_context, provide_user_context};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    // Provide user and company context at the app level so all pages can access it
    provide_user_context();
    provide_company_context();

    view! {
        <Title text="Baraba - Accounting System"/>

        <AppRouter/>
    }
}
