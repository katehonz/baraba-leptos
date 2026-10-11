use crate::router::AppRouter;
use crate::context::{provide_company_context, provide_user_context};
use leptos::prelude::*;
use leptos_meta::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_user_context();
    provide_company_context();

    view! {
        <Title text="Baraba - Accounting System"/>
        <AppRouter/>
    }
}
