# User Company Role - връзка потребител-компания-роля
class UserCompanyRole < BaseModel
  table do
    belongs_to user : User
    belongs_to company : Company
    belongs_to role : Role
  end
end
