use leptos::*;

#[component]
pub fn Button(
    #[prop(into, optional)] label: Option<MaybeSignal<String>>,
    #[prop(into, optional)] onclick: Option<Callback<ev::MouseEvent>>,
    #[prop(into, optional)] class: Option<MaybeSignal<String>>,
    #[prop(into, optional)] disabled: Option<MaybeSignal<bool>>,
    children: Children,
) -> impl IntoView {
    let base_class = "btn";
    let class = move || {
        format!("{} {}", base_class, class.as_ref().map(|c| c.get()).unwrap_or_default())
    };
    
    view! {
        <button
            class=class
            on:click=move |e| {
                if let Some(cb) = &onclick {
                    cb.call(e);
                }
            }
            disabled=disabled.map(|d| d.get()).unwrap_or(false)
        >
            {label.as_ref().map(|l| view! { <span>{l.get()}</span> })}
            {children()}
        </button>
    }
}

#[component]
pub fn Input(
    #[prop(into, optional)] label: Option<MaybeSignal<String>>,
    #[prop(into, optional)] placeholder: Option<MaybeSignal<String>>,
    #[prop(into, optional)] r#type: Option<MaybeSignal<String>>,
    #[prop(into)] value: RwSignal<String>,
    #[prop(into, optional)] class: Option<MaybeSignal<String>>,
    #[prop(into, optional)] disabled: Option<MaybeSignal<bool>>,
) -> impl IntoView {
    let base_class = "form-control";
    let class = move || {
        format!("{} {}", base_class, class.as_ref().map(|c| c.get()).unwrap_or_default())
    };
    
    view! {
        <div class="form-group">
            {label.as_ref().map(|l| view! { <label>{l.get()}</label> })}
            <input
                type=r#type.map(|t| t.get()).unwrap_or_else(|| "text".to_string())
                class=class
                placeholder=placeholder.map(|p| p.get()).unwrap_or_default()
                prop:value=move || value.get()
                on:input=move |e| value.set(event_target_value(&e))
                disabled=disabled.map(|d| d.get()).unwrap_or(false)
            />
        </div>
    }
}

#[component]
pub fn Select<T: 'static + PartialEq + Clone + ToString>(
    #[prop(into, optional)] label: Option<MaybeSignal<String>>,
    #[prop(into)] options: MaybeSignal<Vec<(String, T)>>,
    #[prop(into)] value: RwSignal<Option<T>>,
    #[prop(into, optional)] class: Option<MaybeSignal<String>>,
) -> impl IntoView {
    let base_class = "form-control";
    let class = move || {
        format!("{} {}", base_class, class.as_ref().map(|c| c.get()).unwrap_or_default())
    };
    let options_for_change = options.clone();

    view! {
        <div class="form-group">
            {label.as_ref().map(|l| view! { <label>{l.get()}</label> })}
            <select
                class=class
                on:change=move |e| {
                    let selected = event_target_value(&e);
                    let opts = options_for_change.get();
                    if let Some((_, val)) = opts.iter().find(|(id, _)| id == &selected) {
                        value.set(Some(val.clone()));
                    }
                }
            >
                <option value="">"Select..."</option>
                {move || {
                    options.get().into_iter().map(|(id, val)| {
                        let selected = value.get().map(|v| v == val).unwrap_or(false);
                        let id_clone = id.clone();
                        view! {
                            <option value=id selected=selected>
                                {id_clone}
                            </option>
                        }
                    }).collect_view()
                }}
            </select>
        </div>
    }
}

#[component]
pub fn Loading(
    #[prop(into, optional)] message: Option<MaybeSignal<String>>,
) -> impl IntoView {
    view! {
        <div class="loading">
            <div class="spinner"></div>
            <p>{message.map(|m| m.get()).unwrap_or_else(|| "Loading...".to_string())}</p>
        </div>
    }
}

#[component]
pub fn Alert(
    #[prop(into)] variant: String,
    #[prop(into)] message: String,
) -> impl IntoView {
    view! {
        <div class=format!("alert alert-{}", variant)>
            {message}
        </div>
    }
}
