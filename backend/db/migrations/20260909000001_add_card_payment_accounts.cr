class AddCardPaymentAccounts::V20260909000001 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Company) do
      add card_purchase_account_id : Int64?
      add card_sales_account_id : Int64?
    end

    alter table_for(ScannedInvoice) do
      add payment_method : String?
      add payment_method_raw : String?
      add payment_journal_entry_id : Int64?
    end
  end

  def rollback
    alter table_for(ScannedInvoice) do
      remove :payment_method
      remove :payment_method_raw
      remove :payment_journal_entry_id
    end

    alter table_for(Company) do
      remove :card_purchase_account_id
      remove :card_sales_account_id
    end
  end
end
