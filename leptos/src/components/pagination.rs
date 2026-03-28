use leptos::*;

#[derive(Clone, Debug, Default)]
pub struct PaginationInfo {
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
    pub total_pages: i32,
}

#[component]
pub fn Pagination(
    pagination: Signal<PaginationInfo>,
    on_page_change: Callback<i32>,
    #[prop(optional)] on_per_page_change: Option<Callback<i32>>,
) -> impl IntoView {
    let page = move || pagination.get().page;
    let total_pages = move || pagination.get().total_pages;
    let total = move || pagination.get().total;
    let per_page = move || pagination.get().per_page;

    let start_item = move || ((page() - 1) * per_page()) + 1;
    let end_item = move || std::cmp::min(page() * per_page(), total());

    let can_prev = move || page() > 1;
    let can_next = move || page() < total_pages();

    let go_prev = move |_| {
        if can_prev() {
            on_page_change.call(page() - 1);
        }
    };

    let go_next = move |_| {
        if can_next() {
            on_page_change.call(page() + 1);
        }
    };

    let go_first = move |_| {
        on_page_change.call(1);
    };

    let go_last = move |_| {
        on_page_change.call(total_pages());
    };

    // Generate page numbers to show
    let page_numbers = move || {
        let current = page();
        let total = total_pages();
        let mut pages = Vec::new();

        if total <= 7 {
            // Show all pages
            for i in 1..=total {
                pages.push(Some(i));
            }
        } else {
            // Always show first page
            pages.push(Some(1));

            if current > 3 {
                pages.push(None); // Ellipsis
            }

            // Pages around current
            let start = std::cmp::max(2, current - 1);
            let end = std::cmp::min(total - 1, current + 1);
            for i in start..=end {
                pages.push(Some(i));
            }

            if current < total - 2 {
                pages.push(None); // Ellipsis
            }

            // Always show last page
            if total > 1 {
                pages.push(Some(total));
            }
        }

        pages
    };

    view! {
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px; padding: 15px 0;">
            // Info text
            <div style="color: #7f8c8d; font-size: 14px;">
                {move || {
                    let t = total();
                    if t == 0 {
                        "Няма резултати".to_string()
                    } else {
                        format!("Показване {} - {} от {}", start_item(), end_item(), t)
                    }
                }}
            </div>

            // Page controls
            <div style="display: flex; align-items: center; gap: 5px;">
                // First page
                <button
                    style=move || format!(
                        "padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; cursor: {}; background: white; color: {};",
                        if can_prev() { "pointer" } else { "not-allowed" },
                        if can_prev() { "#34495e" } else { "#bdc3c7" }
                    )
                    on:click=go_first
                    disabled=move || !can_prev()
                    title="Първа страница"
                >
                    "«"
                </button>

                // Previous
                <button
                    style=move || format!(
                        "padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; cursor: {}; background: white; color: {};",
                        if can_prev() { "pointer" } else { "not-allowed" },
                        if can_prev() { "#34495e" } else { "#bdc3c7" }
                    )
                    on:click=go_prev
                    disabled=move || !can_prev()
                    title="Предишна"
                >
                    "‹"
                </button>

                // Page numbers
                {move || {
                    page_numbers().into_iter().map(|p| {
                        match p {
                            Some(num) => {
                                let is_current = num == page();
                                let page_num = num;
                                view! {
                                    <button
                                        style=move || format!(
                                            "padding: 8px 12px; border: 1px solid {}; border-radius: 4px; cursor: pointer; background: {}; color: {}; min-width: 40px;",
                                            if is_current { "#3498db" } else { "#ddd" },
                                            if is_current { "#3498db" } else { "white" },
                                            if is_current { "white" } else { "#34495e" }
                                        )
                                        on:click=move |_| on_page_change.call(page_num)
                                    >
                                        {num}
                                    </button>
                                }.into_view()
                            }
                            None => {
                                view! {
                                    <span style="padding: 8px 4px; color: #7f8c8d;">"..."</span>
                                }.into_view()
                            }
                        }
                    }).collect_view()
                }}

                // Next
                <button
                    style=move || format!(
                        "padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; cursor: {}; background: white; color: {};",
                        if can_next() { "pointer" } else { "not-allowed" },
                        if can_next() { "#34495e" } else { "#bdc3c7" }
                    )
                    on:click=go_next
                    disabled=move || !can_next()
                    title="Следваща"
                >
                    "›"
                </button>

                // Last page
                <button
                    style=move || format!(
                        "padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; cursor: {}; background: white; color: {};",
                        if can_next() { "pointer" } else { "not-allowed" },
                        if can_next() { "#34495e" } else { "#bdc3c7" }
                    )
                    on:click=go_last
                    disabled=move || !can_next()
                    title="Последна страница"
                >
                    "»"
                </button>
            </div>

            // Per page selector
            {move || {
                if let Some(callback) = on_per_page_change {
                    view! {
                        <div style="display: flex; align-items: center; gap: 8px;">
                            <span style="color: #7f8c8d; font-size: 14px;">"На страница:"</span>
                            <select
                                style="padding: 8px; border: 1px solid #ddd; border-radius: 4px; background: white;"
                                on:change=move |ev| {
                                    let val = event_target_value(&ev).parse::<i32>().unwrap_or(25);
                                    callback.call(val);
                                }
                            >
                                <option value="10" selected=move || per_page() == 10>"10"</option>
                                <option value="25" selected=move || per_page() == 25>"25"</option>
                                <option value="50" selected=move || per_page() == 50>"50"</option>
                                <option value="100" selected=move || per_page() == 100>"100"</option>
                            </select>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
        </div>
    }
}
