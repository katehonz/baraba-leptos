Универсален експорт на счетоводни контировки от CONTROLISY в XML, версия 2/04.01.2025

1	2	3	4	5	6	Атрибути	Описание
<?xml version="1.0" encoding="utf-8"?>
<ExportedData>
	<Contractors>						Списък на контрагентите, които участват в експорта.
		<Contractor
						contractorName	Контрагент име
						contractorEIK	Контрагент ЕИК
						contractorVATNumber	Контрагент ДДС номер
						contractorInsideNumber	Вътрешен номер контрагент
						ca_contractorId	ID на контрагент в CONTROLISY
		/>
	</Contractors>
	<Documents>
		<Document
						accountingMonth	Счетоводен месец - YYYY-MM-01
						vatMonth	ДДС месец - YYYY-MM-01
						maturity	Падеж - YYYY-MM-01
						docIdent	Идентификатор на вид документ според изискванията на ППЗДДС. Например: 01. Виж шийт Списъци.
						documentDate	Дата документ - YYYY-MM-01
						documentNumber	Номер документ
						ca_docId	ID на документа в CONTROLISY
						reason	Основание за ДДС дневниците
						remark	Забележка. Не участва в ДДС дневниците.
						netAmountBGN	Обща сума по документа в лева без ДДС
						vatAmountBGN	Обща сума по документа на ДДС в лева
						totalAmountBGN	Обща сума по документа в лева с ДДС
						ca_vatOperationID	ID на ДДС операцията в CONTROLISY за целия документ
						ca_docTypeID	ID на вид документ в CONTROLISY
						ca_contractorId	ID на контрагент в CONTROLISY
		>
			<Accountings>
				<Accounting			Сложните контировки се представят като отделни прости контировки в рамките на Accountings
						amountBGN	Стойност на контировката в лева
						ca_vatOperationID	ID на ДДС операцията в CONTROLISY за конкретната контировка
				>
					<AccountingDetail		В рамките на Accounting винаги има 1 AccountingDetail за direction="Debit" и 1 за direction="Credit"
						direction	Debit или Credit
						currency	Валута. Например EUR, USD и т.н. Ако валутата е лева, остава се празно.
						currencyAmount	Количество валута. Оставя се празно, ако валутата е лев.
						unit	Мерна единица
						quantity	Количество
						accountNumber	Номер счетоводна сметка
						accountName	Наименование счетоводна сметка
						accountItem1	Партида 1
						accountItem2	Партида 2
						accountItem3	Партида 3
						accountItem4	Партида 4
						contractorName	Контрагент име, ако счетоводната сметка е свързана с контрагент
						contractorEIK	Контрагент ЕИК, ако счетоводната сметка е свързана с контрагент
						contractorVATNumber	Контрагент ДДС номер, ако счетоводната сметка е свързана с контрагент
					/>
				</Accounting>
			</Accountings>
			<VATData
						vatRegister	Идентификатор за ДДС регистъра. 1 за Покупка, 2 за Продажба или празно.
						contractorName	Контрагент име
						contractorVATNumber	Контрагент ДДС номер
			>
				<VAT			Обикновено се подава в рамките на един таг, но при смесени ДДС сделки е възможно да станат 2 тага.
						taxBase	Данъчна основа в лева
						vatRate	ДДС ставка, изписва се без знака за %, например 20.
						vatAmountBGN	Стойност на ДДС в лева
						ca_vatOperationID	ID на ДДС операцията в CONTROLISY
						vatOperationIden	Идентификатор за ДДС операцията. Виж шийт Списъци.
						vatOperationIdenName	Наименование на идентификатора на ДДС операцията. Виж шийт Списъци.
						vatOperationName	Наименование на ДДС операцията, заложено в CONTROLISY.
						vatOperationAdditionalIden	Идентификатор на допълнителната ДДС операция. Виж шийт Списъци.
						vatOperationAdditionalName	Наименование на допълнителната ДДС операция. Виж шийт Списъци.
						vatOperationDetail	Допълнителни данни с цел указване на Приложение 2, част 1 и 2. Виж шийт Списъци.
						vatOperationIdent1	Допълнителен свободен идентификатор за ДДС операция
						vatOperationIdent2	Допълнителен свободен идентификатор за ДДС операция
						vatOperationIdent3	Допълнителен свободен идентификатор за ДДС операция
						vatOperationIdent4	Допълнителен свободен идентификатор за ДДС операция
				/>
			<RelDoc				Обикновено се използва за указване на фра, по която е извършено плащането, ако текущият документ е касово или банково плащане
						ca_relDocumentId	ID на свързания документ в CONTROLISY
						relDocumentNumber	Номер на свързан документ
						relDocumentDate	Дата на свързан документ
						contractorName	Контрагент име по свързания документ
						contractorEIK	Контрагент ЕИК по свързания документ
						contractorVATNumber	Контрагент ДДС номер по свързания документ
			/>
		</Document>
	</Documents>
</ExportedData>
