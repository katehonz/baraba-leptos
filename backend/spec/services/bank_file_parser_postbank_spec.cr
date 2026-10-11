require "../spec_helper"

describe BankFileParser do
  it "parses the 01-2026 Юробанк/Пощенска export with BookingDate 2026-01-05 not today" do
    path = File.expand_path("../../../banki/postbank-bg/01--2026--Движения_По_Сметка_BG93TEST-XX-01.xml", __DIR__)
    pending! "sample file missing" unless File.exists?(path)

    result = BankFileParser.parse(File.read(path))
    result.bank_format.should eq("POSTBANK")
    result.account_iban.should eq("BG25TEST00000134567892")
    result.transactions.size.should be > 10
    test_tx = result.transactions.find { |t| t.description.try(&.includes?("ТЕСТ МАГАЗИН")) }
    test_tx.should_not be_nil
    test_tx.not_nil!.date.should eq(Time.utc(2026, 1, 5))
    test_tx.not_nil!.amount.should eq(-3.82)
    result.transactions.none? { |t| t.date.month == 9 && t.date.day == 11 }.should be_true
  end

  it "parses Eurobank Postbank CustomerReport XML using BookingDate, not today" do
    xml = <<-XML
      <?xml version="1.0" encoding="utf-8"?>
      <CustomerReportAccountsTransactionsHistory>
        <MultipleAccountTransactionItemAPIOutputModel>
          <AccountNumber>BG25TEST00000134567892</AccountNumber>
          <AccountCurrency>BGN</AccountCurrency>
          <TypeOperation>POS трансакция</TypeOperation>
          <FlowDirection>Outbound</FlowDirection>
          <Counterparty></Counterparty>
          <FromToAccount></FromToAccount>
          <TransactionAmount>
            <Value>3.82</Value>
            <CurrencyCode><Code>BGN</Code></CurrencyCode>
            <Amount>3.82</Amount>
          </TransactionAmount>
          <RemittanceInformation>Транзакция с ДК 000000XXXXXX0000,04.01.2026 13:27:34,ТЕСТ МАГАЗИН</RemittanceInformation>
          <BorderoNumber>R0000001</BorderoNumber>
          <BookingDate>
            <DateTimeValue>2026-01-04T00:00:00</DateTimeValue>
            <IsEmptyDate>false</IsEmptyDate>
          </BookingDate>
          <ValueDate>
            <DateTimeValue>2026-01-04T00:00:00</DateTimeValue>
            <IsEmptyDate>false</IsEmptyDate>
          </ValueDate>
        </MultipleAccountTransactionItemAPIOutputModel>
      </CustomerReportAccountsTransactionsHistory>
      XML

    result = BankFileParser.parse(xml)
    result.bank_format.should eq("POSTBANK")
    result.account_iban.should eq("BG25TEST00000134567892")
    result.transactions.size.should eq(1)
    tx = result.transactions.first
    tx.date.year.should eq(2026)
    tx.date.month.should eq(1)
    tx.date.day.should eq(4)
    tx.amount.should eq(-3.82)
    tx.currency.should eq("BGN")
    tx.reference.should eq("R0000001")
  end

  it "parses internet-banking AccountTransactionsHistoryModel BookingDate without timezone" do
    xml = <<-XML
      <?xml version="1.0" encoding="utf-8"?>
      <AccountTransactionsHistory>
        <AccountTransactionsHistoryModel>
          <TransactionAmount><Value>3.82</Value><Currency>EUR</Currency></TransactionAmount>
          <BorderNumber>R0000001</BorderNumber>
          <FlowDirection>Outbound</FlowDirection>
          <TransactionType>POS трансакция</TransactionType>
          <RemittanceInformation>Транзакция с ДК,04.01.2026 13:27:34,ТЕСТ МАГАЗИН</RemittanceInformation>
          <ValueDate>2026-01-05T00:00:00</ValueDate>
          <BookingDate>2026-01-05T00:00:00</BookingDate>
          <AccountNumber>BG25TEST00000134567892</AccountNumber>
        </AccountTransactionsHistoryModel>
      </AccountTransactionsHistory>
      XML

    result = BankFileParser.parse(xml)
    result.transactions.size.should eq(1)
    tx = result.transactions.first
    tx.date.should eq(Time.utc(2026, 1, 5))
    tx.amount.should eq(-3.82)
    tx.currency.should eq("EUR")
  end

  it "does not use today's date when BookingDate has nested DateTimeValue + IsEmptyDate" do
    xml = <<-XML
      <AccountTransactionsHistory>
        <AccountTransactionsHistoryModel>
          <AccountIban>BG00TEST</AccountIban>
          <BookingDate>2025-07-01T00:00:00false</BookingDate>
          <TransactionAmount>
            <Value>10</Value>
            <Currency>EUR</Currency>
          </TransactionAmount>
          <FlowDirection>Inbound</FlowDirection>
          <TransactionType>Test</TransactionType>
          <BorderNumber>1</BorderNumber>
        </AccountTransactionsHistoryModel>
      </AccountTransactionsHistory>
      XML

    result = BankFileParser.parse(xml)
    result.transactions.size.should eq(1)
    tx = result.transactions.first
    tx.date.should eq(Time.utc(2025, 7, 1))
  end
end
