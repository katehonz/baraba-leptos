# Opening Balance - начално салдо за сметка/контрагент/продукт
class OpeningBalance < BaseModel
  table do
    column debit : Float64
    column credit : Float64
    column date : Time
    column quantity : Float64?
    column description : String?
    column balance_type : String = "account"  # account, counterpart, product

    belongs_to account : Account
    belongs_to company : Company
    belongs_to accounting_period : AccountingPeriod?
    belongs_to counterpart : Counterpart?
    belongs_to product : Product?
    belongs_to warehouse : Warehouse?
  end

  # Calculate net balance (debit - credit for active accounts, credit - debit for passive)
  def net_balance : Float64
    debit - credit
  end

  # Check if this is a subledger balance (has counterpart or product)
  def subledger? : Bool
    !counterpart_id.nil? || !product_id.nil?
  end
end
