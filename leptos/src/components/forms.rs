use leptos::*;

#[component]
pub fn Modal(
    #[prop(into)] show: RwSignal<bool>,
    #[prop(into, optional)] title: Option<MaybeSignal<String>>,
    #[prop(into, optional)] size: Option<MaybeSignal<String>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="modal" class:modal-show=move || show.get()>
            <div class=format!("modal-dialog {}", size.map(|s| s.get()).unwrap_or_default())>
                <div class="modal-content">
                    <div class="modal-header">
                        {title.as_ref().map(|t| view! { <h5 class="modal-title">{t.get()}</h5> })}
                        <button class="close" on:click=move |_| show.set(false)>"×"</button>
                    </div>
                    <div class="modal-body">
                        {children()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Card(
    #[prop(into, optional)] title: Option<MaybeSignal<String>>,
    #[prop(into, optional)] class: Option<MaybeSignal<String>>,
    children: Children,
) -> impl IntoView {
    let base_class = "card";
    let class = move || {
        format!("{} {}", base_class, class.as_ref().map(|c| c.get()).unwrap_or_default())
    };
    
    view! {
        <div class=class>
            {title.as_ref().map(|t| view! {
                <div class="card-header">
                    <h5>{t.get()}</h5>
                </div>
            })}
            <div class="card-body">
                {children()}
            </div>
        </div>
    }
}
