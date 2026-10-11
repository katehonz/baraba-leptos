class AddDividendDistributionAndLinkBeneficialOwner::V20260201093353 < Avram::Migrator::Migration::V1
  def migrate
    # Разпределение на дивиденти - основен документ за разпределяне на дивиденти към собственици
    create table_for(DividendDistribution) do
      primary_key id : Int64
      add_timestamps

      add year : Int32                           # Финансова година
      add total_amount : Float64                 # Обща сума за разпределяне
      add decision_date : Time                   # Дата на решение (протокол на ОС)
      add decision_number : String?              # Номер на протокол
      add status : String, default: "draft"     # draft, approved, partially_paid, paid
      add notes : String?
      add_belongs_to company : Company, on_delete: :cascade
    end

    # Добавяме връзка към beneficial_owner в dividends таблицата
    alter table_for(Dividend) do
      add_belongs_to beneficial_owner : BeneficialOwner?, on_delete: :cascade
      add_belongs_to dividend_distribution : DividendDistribution?, on_delete: :cascade
      add is_paid : Bool, default: false
    end
  end

  def rollback
    alter table_for(Dividend) do
      remove_belongs_to :beneficial_owner
      remove_belongs_to :dividend_distribution
      remove :is_paid
    end

    drop table_for(DividendDistribution)
  end
end
