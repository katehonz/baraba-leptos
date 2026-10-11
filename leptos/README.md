# Baraba Frontend

Leptos 0.8 CSR (Client-Side Rendering) frontend за счетоводната система Baraba.

## Билд

Билдът става чрез Docker (Trunk-based CSR):

```bash
docker compose build frontend
docker compose up -d frontend
```

## Технологии

- **Framework:** Leptos 0.8
- **Bundler:** Trunk
- **Target:** wasm32-unknown-unknown
- **HTTP:** gloo-net
- **State:** Leptos Signals (RwSignal, Memo, Effect)
- **Styling:** Inline CSS + Tailwind CSS

## Структура

```
src/
├── main.rs            # SSR entry point (не се ползва при CSR)
├── lib.rs             # WASM entry point (hydrate mode)
├── app.rs             # Root компонент
├── router.rs          # Маршрутизация (leptos_router)
├── api.rs             # API клиент (gloo-net)
├── models.rs          # Serde модели
├── i18n/              # Интернационализация (BG/EN)
├── context/           # React-like context providers
├── components/        # Споделени компоненти
│   ├── layout.rs      # Главен layout с навигация
│   ├── common.rs      # Button, Input, Select, Pagination
│   ├── forms.rs       # Формови компоненти
│   └── pagination.rs  # Пагинация
├── pages/             # 32+ страници
└── stores/            # State management stores
```

## Бележки за Leptos 0.8

- `use leptos::prelude::*` заменя `use leptos::*`
- `RwSignal::new()` заменя `create_rw_signal()`
- `Effect::new()` заменя `create_effect()`
- `Callback::run()` заменя `Callback::call()`
- `NodeRef::<html::Input>::new()` заменя `create_node_ref::<html::Input>()()`
- `.into_any()` за if/else branching с различни HTML елементи
- `&String` не имплементира `IntoRender` - използва се `.clone()`
