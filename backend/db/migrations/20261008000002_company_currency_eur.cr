# Bulgaria's currency is EUR from 2026. Companies still stored as BGN
# keep their books in the euro amounts already posted, so EUR documents
# that were converted with the fixed 1.95583 rate go back to rate 1.
class CompanyCurrencyEur::V20261008000002 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      UPDATE invoices
      SET exchange_rate = 1.0
      WHERE currency = 'EUR'
        AND abs(exchange_rate - 1.95583) < 0.000001
        AND company_id IN (SELECT id FROM companies WHERE currency = 'BGN')
    SQL

    execute <<-SQL
      UPDATE companies
      SET currency = 'EUR'
      WHERE currency = 'BGN' OR currency IS NULL OR btrim(currency) = ''
    SQL
  end

  def rollback
    execute "SELECT 1"
  end
end
