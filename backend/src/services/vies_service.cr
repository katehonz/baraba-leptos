require "http/client"
require "xml"

# Service to validate VAT numbers via EU VIES
class ViesService
  VIES_URL = "http://ec.europa.eu/taxation_customs/vies/services/checkVatService"

  struct ViesResult
    property valid : Bool
    property name : String
    property address : String
    property country_code : String
    property vat_number : String
    property error : String?

    def initialize(
      @valid = false,
      @name = "",
      @address = "",
      @country_code = "",
      @vat_number = "",
      @error = nil
    )
    end

    def to_json(json : JSON::Builder)
      json.object do
        json.field "valid", @valid
        json.field "name", @name
        json.field "address", @address
        json.field "country_code", @country_code
        json.field "vat_number", @vat_number
        json.field "error", @error if @error
      end
    end
  end

  def self.check_vat(full_vat_number : String) : ViesResult
    # VAT number must be at least 3 characters (2 for country code + 1 digit)
    if full_vat_number.size < 3
      return ViesResult.new(error: "VAT number too short")
    end

    # Extract country code and number
    country_code = full_vat_number[0, 2].upcase
    vat_number = full_vat_number[2..]

    check_vat(country_code, vat_number)
  end

  def self.check_vat(country_code : String, vat_number : String) : ViesResult
    soap_envelope = <<-XML
      <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:urn="urn:ec.europa.eu:taxud:vies:services:checkVat:types">
         <soapenv:Header/>
         <soapenv:Body>
            <urn:checkVat>
               <urn:countryCode>#{country_code}</urn:countryCode>
               <urn:vatNumber>#{vat_number}</urn:vatNumber>
            </urn:checkVat>
         </soapenv:Body>
      </soapenv:Envelope>
    XML

    begin
      uri = URI.parse(VIES_URL)
      client = HTTP::Client.new(uri)
      client.connect_timeout = 10.seconds
      client.read_timeout = 30.seconds

      response = client.post(
        uri.path.not_nil!,
        headers: HTTP::Headers{
          "Content-Type" => "text/xml;charset=UTF-8",
          "SOAPAction"   => "",
        },
        body: soap_envelope
      )

      if response.status_code != 200
        return ViesResult.new(
          country_code: country_code,
          vat_number: vat_number,
          error: "VIES service returned status #{response.status_code}"
        )
      end

      parse_response(response.body, country_code, vat_number)
    rescue ex
      ViesResult.new(
        country_code: country_code,
        vat_number: vat_number,
        error: "VIES service error: #{ex.message}"
      )
    end
  end

  private def self.parse_response(xml_body : String, country_code : String, vat_number : String) : ViesResult
    doc = XML.parse(xml_body)

    # Find elements by local name (VIES uses namespaces)
    valid = find_element_text(doc, "valid") == "true"
    name = find_element_text(doc, "name")
    address = find_element_text(doc, "address")

    # Clean up placeholder values
    name = "" if name == "---"
    address = "" if address == "---"

    ViesResult.new(
      valid: valid,
      name: name,
      address: address,
      country_code: country_code,
      vat_number: vat_number
    )
  end

  private def self.find_element_text(doc : XML::Node, local_name : String) : String
    # Search for element with matching local name
    doc.xpath_nodes("//*[local-name()='#{local_name}']").first?.try(&.text) || ""
  end
end
