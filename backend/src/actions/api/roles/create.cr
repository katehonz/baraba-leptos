class Api::Roles::Create < ApiAction
  post "/api/roles" do
    SaveRole.create(params) do |operation, role|
      if role
        json({success: true, data: {id: role.id, name: role.name, description: role.description}})
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
