class BankAccount < BaseModel
  table do
    column name : String
    column iban : String
    column bic : String?
    column currency : String
    column bank_name : String?
    column integration_type : String = "manual"  # manual, salt_edge, wise

    belongs_to company : Company
    belongs_to gl_account : Account?
    belongs_to buffer_account : Account?

    # Salt Edge Integration
    column salt_edge_connection_id : String?
    column salt_edge_account_id : String?
    column salt_edge_provider_code : String?
    column salt_edge_last_sync : Time?
    column is_salt_edge_connected : Bool = false

    # Wise Integration
    column wise_account_id : String?
    column wise_balance_type : String?
    column wise_last_sync : Time?
    column is_wise_connected : Bool = false
  end
end
