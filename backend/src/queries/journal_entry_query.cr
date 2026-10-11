class JournalEntryQuery < JournalEntry::BaseQuery
  # Search description, reference, document number, and counterpart (name/EIK/VAT).
  def search(term : String?) : self
    return self if term.nil? || term.empty?

    pattern = "%#{term}%"
    sql = <<-SQL
      (
        journal_entries.description ILIKE ?
        OR journal_entries.reference ILIKE ?
        OR journal_entries.document_number ILIKE ?
        OR journal_entries.counterpart_id IN (
          SELECT counterparts.id FROM counterparts
          WHERE counterparts.company_id = journal_entries.company_id
            AND (
              counterparts.name ILIKE ?
              OR counterparts.eik ILIKE ?
              OR counterparts.vat_number ILIKE ?
            )
        )
      )
    SQL
    where(sql, pattern, pattern, pattern, pattern, pattern, pattern)
  end
end
