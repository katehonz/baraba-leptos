require "xml"

# Bank file parser service - parses various bank statement formats
class BankFileParser
  # Parsed transaction data
  class ParsedTransaction
    property date : Time
    property amount : Float64
    property currency : String
    property description : String?
    property contra_account : String?
    property contra_name : String?
    property reference : String?

    def initialize(
      @date : Time,
      @amount : Float64,
      @currency : String = "EUR",
      @description : String? = nil,
      @contra_account : String? = nil,
      @contra_name : String? = nil,
      @reference : String? = nil
    )
    end
  end

  # Parse result
  class ParseResult
    property transactions : Array(ParsedTransaction)
    property bank_format : String
    property account_iban : String?
    property account_currency : String?
    property opening_balance : Float64?
    property closing_balance : Float64?
    property period_from : Time?
    property period_to : Time?

    def initialize(
      @transactions : Array(ParsedTransaction) = [] of ParsedTransaction,
      @bank_format : String = "UNKNOWN",
      @account_iban : String? = nil,
      @account_currency : String? = nil,
      @opening_balance : Float64? = nil,
      @closing_balance : Float64? = nil,
      @period_from : Time? = nil,
      @period_to : Time? = nil
    )
    end
  end

  # Auto-detect and parse file
  def self.parse(content : String) : ParseResult
    # Try to detect format
    if content.includes?("<STATEMENT>") && content.includes?("<POST_DATE>")
      return parse_obb(content)
    elsif content.includes?("camt.053") || content.includes?("BkToCstmrStmt")
      return parse_camt053(content)
    elsif content.includes?("<AccountTransactionsHistory")
      return parse_postbank(content)
    elsif content.starts_with?(":20:") || content.includes?(":60F:") || content.includes?(":61:")
      return parse_mt940(content)
    elsif content.includes?(",") && (content.includes?("IBAN") || content.includes?("Дата"))
      return parse_csv(content)
    else
      raise "Непознат формат на файла"
    end
  end

  # Parse OBB (KBC Bulgaria) XML format
  def self.parse_obb(content : String) : ParseResult
    result = ParseResult.new(bank_format: "OBB")
    doc = XML.parse(content)

    # Get account info
    if iban_node = doc.xpath_node("//IBAN_S")
      result.account_iban = iban_node.content.strip
    end

    # Get balances
    if open_bal = doc.xpath_node("//OPEN_BALANCE")
      result.opening_balance = parse_obb_amount(open_bal.content)
    end
    if close_bal = doc.xpath_node("//CLOSE_BALANCE")
      result.closing_balance = parse_obb_amount(close_bal.content)
    end

    # Get period
    if from_date = doc.xpath_node("//FROM_ST_DATE")
      result.period_from = parse_obb_date(from_date.content)
    end
    if to_date = doc.xpath_node("//TILL_ST_DATE")
      result.period_to = parse_obb_date(to_date.content)
    end

    result.account_currency = "BGN"

    # Parse transactions
    doc.xpath_nodes("//TRANSACTION").each do |tx_node|
      tx = parse_obb_transaction(tx_node)
      result.transactions << tx if tx
    end

    result
  end

  private def self.parse_obb_transaction(node : XML::Node) : ParsedTransaction?
    post_date_str = node.xpath_node("POST_DATE").try(&.content.strip)
    return nil if post_date_str.nil? || post_date_str.empty?

    date = parse_obb_date(post_date_str)

    # OBB has separate debit/credit fields
    amount_d = node.xpath_node("AMOUNT_D").try(&.content.strip)
    amount_c = node.xpath_node("AMOUNT_C").try(&.content.strip)

    amount = 0.0
    if amount_d && !amount_d.empty?
      amount = -parse_obb_amount(amount_d) # Debit = outgoing = negative
    elsif amount_c && !amount_c.empty?
      amount = parse_obb_amount(amount_c) # Credit = incoming = positive
    end

    return nil if amount == 0.0

    tr_name = node.xpath_node("TR_NAME").try(&.content.strip)
    rem_i = node.xpath_node("REM_I").try(&.content.strip)
    rem_ii = node.xpath_node("REM_II").try(&.content.strip)
    description = [tr_name, rem_i, rem_ii].compact.reject(&.empty?).join(" - ")

    contra_name = node.xpath_node("NAME_R").try(&.content.strip)
    contra_account = node.xpath_node("IBAN_R").try(&.content.strip)
    reference = node.xpath_node("REFERENCE").try(&.content.strip)

    ParsedTransaction.new(
      date: date,
      amount: amount,
      currency: "BGN",
      description: description.empty? ? nil : description,
      contra_account: contra_account,
      contra_name: contra_name,
      reference: reference
    )
  end

  private def self.parse_obb_date(date_str : String) : Time
    # Format: DD/MM/YYYY
    parts = date_str.split("/")
    Time.utc(parts[2].to_i, parts[1].to_i, parts[0].to_i)
  end

  private def self.parse_obb_amount(amount_str : String) : Float64
    # Format: 1,234.56 (comma as thousands separator, dot as decimal)
    amount_str.gsub(",", "").to_f
  end

  # Parse ISO 20022 camt.053 format (Paysera, Revolut, etc.)
  def self.parse_camt053(content : String) : ParseResult
    result = ParseResult.new(bank_format: "CAMT053")
    doc = XML.parse(content)

    # Remove namespace for easier parsing
    # Get account info
    if iban_node = doc.xpath_node("//*[local-name()='IBAN']")
      result.account_iban = iban_node.content.strip
    end
    if ccy_node = doc.xpath_node("//*[local-name()='Acct']/*[local-name()='Ccy']")
      result.account_currency = ccy_node.content.strip
    end

    # Get balances
    doc.xpath_nodes("//*[local-name()='Bal']").each do |bal_node|
      bal_type = bal_node.xpath_node("*[local-name()='Tp']/*[local-name()='CdOrPrtry']/*[local-name()='Cd']").try(&.content.strip)
      bal_amt = bal_node.xpath_node("*[local-name()='Amt']").try(&.content.strip.to_f)
      bal_ind = bal_node.xpath_node("*[local-name()='CdtDbtInd']").try(&.content.strip)

      if bal_amt && bal_ind == "DBIT"
        bal_amt = -bal_amt
      end

      case bal_type
      when "OPBD"
        result.opening_balance = bal_amt
      when "CLBD"
        result.closing_balance = bal_amt
      end
    end

    # Get period
    if from_dt = doc.xpath_node("//*[local-name()='FrToDt']/*[local-name()='FrDtTm']")
      result.period_from = Time.parse_iso8601(from_dt.content.strip) rescue nil
    end
    if to_dt = doc.xpath_node("//*[local-name()='FrToDt']/*[local-name()='ToDtTm']")
      result.period_to = Time.parse_iso8601(to_dt.content.strip) rescue nil
    end

    # Parse transactions (Ntry elements)
    doc.xpath_nodes("//*[local-name()='Ntry']").each do |entry_node|
      tx = parse_camt053_entry(entry_node, result.account_currency || "EUR")
      result.transactions << tx if tx
    end

    result
  end

  private def self.parse_camt053_entry(node : XML::Node, default_currency : String) : ParsedTransaction?
    # Amount
    amt_node = node.xpath_node("*[local-name()='Amt']")
    return nil unless amt_node

    amount = amt_node.content.strip.to_f
    currency = amt_node["Ccy"]? || default_currency

    # Credit/Debit indicator
    cd_ind = node.xpath_node("*[local-name()='CdtDbtInd']").try(&.content.strip)
    if cd_ind == "DBIT"
      amount = -amount
    end

    # Date - BookgDt or ValDt
    date_str = node.xpath_node("*[local-name()='BookgDt']/*[local-name()='Dt']").try(&.content.strip)
    date_str ||= node.xpath_node("*[local-name()='ValDt']/*[local-name()='Dt']").try(&.content.strip)
    return nil unless date_str

    date = Time.parse(date_str, "%Y-%m-%d", Time::Location::UTC) rescue Time.utc

    # Description from multiple possible locations
    descriptions = [] of String

    # Unstructured remittance info
    node.xpath_nodes(".//*[local-name()='Ustrd']").each do |ustrd|
      text = ustrd.content.strip
      descriptions << text unless text.empty?
    end

    # Additional entry info
    if addtl = node.xpath_node("*[local-name()='AddtlNtryInf']")
      text = addtl.content.strip
      descriptions << text unless text.empty?
    end

    description = descriptions.uniq.join(" | ")

    # Counterparty info
    contra_name : String? = nil
    contra_account : String? = nil

    # Try related parties
    if cd_ind == "DBIT"
      # For debits, look for Cdtr (creditor = recipient)
      contra_name = node.xpath_node(".//*[local-name()='Cdtr']/*[local-name()='Nm']").try(&.content.strip)
      contra_account = node.xpath_node(".//*[local-name()='CdtrAcct']/*[local-name()='Id']/*[local-name()='IBAN']").try(&.content.strip)
    else
      # For credits, look for Dbtr (debtor = sender)
      contra_name = node.xpath_node(".//*[local-name()='Dbtr']/*[local-name()='Nm']").try(&.content.strip)
      contra_account = node.xpath_node(".//*[local-name()='DbtrAcct']/*[local-name()='Id']/*[local-name()='IBAN']").try(&.content.strip)
    end

    # Reference
    reference = node.xpath_node("*[local-name()='NtryRef']").try(&.content.strip)
    reference ||= node.xpath_node(".//*[local-name()='EndToEndId']").try(&.content.strip)

    ParsedTransaction.new(
      date: date,
      amount: amount,
      currency: currency,
      description: description.empty? ? nil : description,
      contra_account: contra_account,
      contra_name: contra_name,
      reference: reference
    )
  end

  # Parse Postbank Bulgaria XML format
  def self.parse_postbank(content : String) : ParseResult
    result = ParseResult.new(bank_format: "POSTBANK")
    doc = XML.parse(content)

    # Get account info from first model
    if first_tx = doc.xpath_node("//AccountTransactionsHistoryModel")
      if iban_node = first_tx.xpath_node("AccountIban")
        result.account_iban = iban_node.content.strip
      end
    end

    # Parse transactions
    doc.xpath_nodes("//AccountTransactionsHistoryModel").each do |tx_node|
      tx = parse_postbank_transaction(tx_node)
      result.transactions << tx if tx
    end

    result
  end

  private def self.parse_postbank_transaction(node : XML::Node) : ParsedTransaction?
    # Date
    date_str = node.xpath_node("BookingDate").try(&.content.strip)
    return nil unless date_str

    date = Time.parse_iso8601(date_str) rescue Time.utc

    # Amount and currency
    amount_node = node.xpath_node("TransactionAmount")
    return nil unless amount_node

    amount = amount_node.xpath_node("Value").try(&.content.strip.to_f) || 0.0
    currency = amount_node.xpath_node("Currency").try(&.content.strip) || "EUR"

    # Flow direction
    flow = node.xpath_node("FlowDirection").try(&.content.strip)
    if flow == "Outbound"
      amount = -amount
    end

    # Description
    tx_type = node.xpath_node("TransactionType").try(&.content.strip)
    remittance = node.xpath_node("RemittanceInformation").try(&.content.strip)
    description = [tx_type, remittance].compact.reject(&.empty?).join(" - ")

    # Counterparty
    contra_account = node.xpath_node("CorrespondentAccount").try(&.content.strip)
    contra_name = node.xpath_node("CorrespondentName").try(&.content.strip)

    # Reference
    reference = node.xpath_node("BorderNumber").try(&.content.strip)

    ParsedTransaction.new(
      date: date,
      amount: amount,
      currency: currency,
      description: description.empty? ? nil : description,
      contra_account: contra_account,
      contra_name: contra_name,
      reference: reference
    )
  end

  # Parse MT940 SWIFT format (Unicredit, etc.)
  def self.parse_mt940(content : String) : ParseResult
    result = ParseResult.new(bank_format: "MT940")

    lines = content.split("\n").map(&.strip)
    current_tx : ParsedTransaction? = nil

    lines.each do |line|
      case line
      when /^:25:(.+)/
        # Account identification
        result.account_iban = $1.strip
      when /^:60F:([CD])(\d{6})([A-Z]{3})([\d,\.]+)/
        # Opening balance: C/D + Date (YYMMDD) + Currency + Amount
        result.account_currency = $3
        result.opening_balance = parse_mt940_amount($4)
        result.opening_balance = -result.opening_balance.not_nil! if $1 == "D"
      when /^:62F:([CD])(\d{6})([A-Z]{3})([\d,\.]+)/
        # Closing balance
        result.closing_balance = parse_mt940_amount($4)
        result.closing_balance = -result.closing_balance.not_nil! if $1 == "D"
      when /^:61:(\d{6})(\d{4})?([CD])([\d,\.]+)/
        # Transaction line: Date + Optional booking date + C/D + Amount
        # Save previous transaction
        result.transactions << current_tx if current_tx

        date_str = $1
        year = 2000 + date_str[0, 2].to_i
        month = date_str[2, 2].to_i
        day = date_str[4, 2].to_i
        date = Time.utc(year, month, day)

        amount = parse_mt940_amount($4)
        amount = -amount if $3 == "D"

        current_tx = ParsedTransaction.new(
          date: date,
          amount: amount,
          currency: result.account_currency || "EUR"
        )
      when /^:86:(.+)/
        # Transaction details
        if current_tx
          current_tx.description = $1.strip
        end
      end
    end

    # Don't forget last transaction
    result.transactions << current_tx if current_tx

    result
  end

  private def self.parse_mt940_amount(amount_str : String) : Float64
    # MT940 uses comma as decimal separator
    amount_str.gsub(",", ".").to_f
  end

  # Parse CSV format (CCBank, generic)
  def self.parse_csv(content : String) : ParseResult
    result = ParseResult.new(bank_format: "CSV")

    lines = content.split("\n").map(&.strip).reject(&.empty?)
    return result if lines.empty?

    # Try to detect delimiter and header
    delimiter = content.includes?(";") ? ";" : ","
    header = lines.first.split(delimiter).map(&.strip.downcase)

    # Find column indices
    date_col = header.index { |h| h.includes?("дата") || h.includes?("date") } || 0
    amount_col = header.index { |h| h.includes?("сума") || h.includes?("amount") } || 1
    desc_col = header.index { |h| h.includes?("описание") || h.includes?("description") || h.includes?("основание") }
    contra_col = header.index { |h| h.includes?("получател") || h.includes?("наредител") || h.includes?("контрагент") }
    ref_col = header.index { |h| h.includes?("референция") || h.includes?("reference") }

    # Parse data rows
    lines[1..].each do |line|
      cols = line.split(delimiter).map(&.strip)
      next if cols.size <= amount_col

      # Parse date (try multiple formats)
      date_str = cols[date_col]
      date = parse_flexible_date(date_str)
      next unless date

      # Parse amount
      amount_str = cols[amount_col].gsub(/[^\d,.\-]/, "").gsub(",", ".")
      amount = amount_str.to_f rescue 0.0
      next if amount == 0.0

      description = desc_col ? cols[desc_col]? : nil
      contra_name = contra_col ? cols[contra_col]? : nil
      reference = ref_col ? cols[ref_col]? : nil

      result.transactions << ParsedTransaction.new(
        date: date,
        amount: amount,
        currency: "BGN",
        description: description,
        contra_name: contra_name,
        reference: reference
      )
    end

    result
  end

  private def self.parse_flexible_date(date_str : String) : Time?
    # Try various date formats
    formats = [
      "%Y-%m-%d",
      "%d.%m.%Y",
      "%d/%m/%Y",
      "%d-%m-%Y",
      "%Y/%m/%d",
    ]

    formats.each do |fmt|
      begin
        return Time.parse(date_str.strip, fmt, Time::Location::UTC)
      rescue
        next
      end
    end

    nil
  end
end
