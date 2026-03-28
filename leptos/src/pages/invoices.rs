use leptos::*;
use crate::components::Layout;

#[component]
pub fn Invoices() -> impl IntoView {
    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <h1>"Фактури"</h1>
                <button style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;">
                    "+ Нова фактура"
                </button>
            </div>

            <div style="background: white; border-radius: 8px; margin-top: 20px; overflow: hidden;">
                <table style="width: 100%; border-collapse: collapse;">
                    <thead style="background: #34495e; color: white;">
                        <tr>
                            <th style="padding: 12px; text-align: left;">"Номер"</th>
                            <th style="padding: 12px; text-align: left;">"Дата"</th>
                            <th style="padding: 12px; text-align: left;">"Контрагент"</th>
                            <th style="padding: 12px; text-align: right;">"Сума"</th>
                            <th style="padding: 12px; text-align: left;">"Статус"</th>
                            <th style="padding: 12px; text-align: center;">"Действия"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td colspan="6" style="padding: 40px; text-align: center; color: #7f8c8d;">
                                "Няма добавени фактури"
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </Layout>
    }
}
