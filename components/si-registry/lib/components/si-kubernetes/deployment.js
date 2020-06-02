"use strict";

var _registry = require("../../registry");

_registry.registry.componentAndEntity({
  typeName: "kubernetesDeployment",
  displayTypeName: "Kubernetes Deployment Object",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options: function options(c) {
    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount"
    });
    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes"
    }); // Constraints

    c.constraints.addEnum({
      name: "kubernetesVersion",
      label: "Kubernetes Version",
      options: function options(p) {
        p.variants = ["v1.12", "v1.13", "v1.14", "v1.15"];
      }
    }); // Properties

    c.properties.addObject({
      name: "kubernetesObject",
      label: "Kubernetes Object",
      options: function options(p) {
        p.relationships.updates({
          partner: {
            typeName: "kubernetesDeploymentEntity",
            names: ["properties", "kubernetesObjectYaml"]
          }
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesDeploymentEntity",
            names: ["properties", "kubernetesObjectYaml"]
          }
        });
        p.properties.addText({
          name: "apiVersion",
          label: "API Version",
          options: function options(p) {
            p.required = true;
          }
        });
        p.properties.addText({
          name: "kind",
          label: "Kind",
          options: function options(p) {
            p.required = true;
            p.baseDefaultValue = "Deployment";
          }
        });
        p.properties.addLink({
          name: "metadata",
          label: "Metadata",
          options: function options(p) {
            p.lookup = {
              typeName: "kubernetesMetadata"
            };
          }
        });
        p.properties.addObject({
          name: "spec",
          label: "Deployment Spec",
          options: function options(p) {
            p.properties.addNumber({
              name: "replicas",
              label: "Replicas",
              options: function options(p) {
                p.numberKind = "int32";
              }
            });
            p.properties.addLink({
              name: "selector",
              label: "Selector",
              options: function options(p) {
                p.lookup = {
                  typeName: "kubernetesSelector"
                };
              }
            });
            p.properties.addLink({
              name: "template",
              label: "Pod Template Spec",
              options: function options(p) {
                p.lookup = {
                  typeName: "kubernetesPodTemplateSpec"
                };
              }
            });
          }
        });
      }
    });
    c.properties.addCode({
      name: "kubernetesObjectYaml",
      label: "Kubernetes Object YAML",
      options: function options(p) {
        p.relationships.updates({
          partner: {
            typeName: "kubernetesDeploymentEntity",
            names: ["properties", "kubernetesObject"]
          }
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesDeploymentEntity",
            names: ["properties", "kubernetesObject"]
          }
        });
        p.language = "yaml";
      }
    });
  }
});
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy9jb21wb25lbnRzL3NpLWt1YmVybmV0ZXMvZGVwbG95bWVudC50cyJdLCJuYW1lcyI6WyJyZWdpc3RyeSIsImNvbXBvbmVudEFuZEVudGl0eSIsInR5cGVOYW1lIiwiZGlzcGxheVR5cGVOYW1lIiwic2lQYXRoTmFtZSIsInNlcnZpY2VOYW1lIiwib3B0aW9ucyIsImMiLCJlbnRpdHkiLCJhc3NvY2lhdGlvbnMiLCJiZWxvbmdzVG8iLCJmcm9tRmllbGRQYXRoIiwiaW50ZWdyYXRpb25TZXJ2aWNlcyIsInB1c2giLCJpbnRlZ3JhdGlvbk5hbWUiLCJpbnRlZ3JhdGlvblNlcnZpY2VOYW1lIiwiY29uc3RyYWludHMiLCJhZGRFbnVtIiwibmFtZSIsImxhYmVsIiwicCIsInZhcmlhbnRzIiwicHJvcGVydGllcyIsImFkZE9iamVjdCIsInJlbGF0aW9uc2hpcHMiLCJ1cGRhdGVzIiwicGFydG5lciIsIm5hbWVzIiwiZWl0aGVyIiwiYWRkVGV4dCIsInJlcXVpcmVkIiwiYmFzZURlZmF1bHRWYWx1ZSIsImFkZExpbmsiLCJsb29rdXAiLCJhZGROdW1iZXIiLCJudW1iZXJLaW5kIiwiYWRkQ29kZSIsImxhbmd1YWdlIl0sIm1hcHBpbmdzIjoiOztBQVFBOztBQUVBQSxtQkFBU0Msa0JBQVQsQ0FBNEI7QUFDMUJDLEVBQUFBLFFBQVEsRUFBRSxzQkFEZ0I7QUFFMUJDLEVBQUFBLGVBQWUsRUFBRSw4QkFGUztBQUcxQkMsRUFBQUEsVUFBVSxFQUFFLGVBSGM7QUFJMUJDLEVBQUFBLFdBQVcsRUFBRSxZQUphO0FBSzFCQyxFQUFBQSxPQUwwQixtQkFLbEJDLENBTGtCLEVBS2Y7QUFDVEEsSUFBQUEsQ0FBQyxDQUFDQyxNQUFGLENBQVNDLFlBQVQsQ0FBc0JDLFNBQXRCLENBQWdDO0FBQzlCQyxNQUFBQSxhQUFhLEVBQUUsQ0FBQyxjQUFELEVBQWlCLGtCQUFqQixDQURlO0FBRTlCVCxNQUFBQSxRQUFRLEVBQUU7QUFGb0IsS0FBaEM7QUFJQUss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LDRCQURIO0FBRVB5QixZQUFBQSxLQUFLLEVBQUUsQ0FBQyxZQUFELEVBQWUsa0JBQWY7QUFGQTtBQURZLFNBQXZCO0FBTUFQLFFBQUFBLENBQUMsQ0FBQ2lCLFFBQUYsR0FBYSxNQUFiO0FBQ0Q7QUFqQmtCLEtBQXJCO0FBbUJEO0FBdEh5QixDQUE1QiIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7XG4gIFByb3BPYmplY3QsXG4gIFByb3BUZXh0LFxuICBQcm9wTGluayxcbiAgUHJvcE51bWJlcixcbiAgUHJvcEVudW0sXG4gIFByb3BDb2RlLFxufSBmcm9tIFwiLi4vLi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi8uLi9yZWdpc3RyeVwiO1xuXG5yZWdpc3RyeS5jb21wb25lbnRBbmRFbnRpdHkoe1xuICB0eXBlTmFtZTogXCJrdWJlcm5ldGVzRGVwbG95bWVudFwiLFxuICBkaXNwbGF5VHlwZU5hbWU6IFwiS3ViZXJuZXRlcyBEZXBsb3ltZW50IE9iamVjdFwiLFxuICBzaVBhdGhOYW1lOiBcInNpLWt1YmVybmV0ZXNcIixcbiAgc2VydmljZU5hbWU6IFwia3ViZXJuZXRlc1wiLFxuICBvcHRpb25zKGMpIHtcbiAgICBjLmVudGl0eS5hc3NvY2lhdGlvbnMuYmVsb25nc1RvKHtcbiAgICAgIGZyb21GaWVsZFBhdGg6IFtcInNpUHJvcGVydGllc1wiLCBcImJpbGxpbmdBY2NvdW50SWRcIl0sXG4gICAgICB0eXBlTmFtZTogXCJiaWxsaW5nQWNjb3VudFwiLFxuICAgIH0pO1xuICAgIGMuZW50aXR5LmludGVncmF0aW9uU2VydmljZXMucHVzaCh7XG4gICAgICBpbnRlZ3JhdGlvbk5hbWU6IFwiYXdzXCIsXG4gICAgICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBcImVrc19rdWJlcm5ldGVzXCIsXG4gICAgfSk7XG5cbiAgICAvLyBDb25zdHJhaW50c1xuICAgIGMuY29uc3RyYWludHMuYWRkRW51bSh7XG4gICAgICBuYW1lOiBcImt1YmVybmV0ZXNWZXJzaW9uXCIsXG4gICAgICBsYWJlbDogXCJLdWJlcm5ldGVzIFZlcnNpb25cIixcbiAgICAgIG9wdGlvbnMocDogUHJvcEVudW0pIHtcbiAgICAgICAgcC52YXJpYW50cyA9IFtcInYxLjEyXCIsIFwidjEuMTNcIiwgXCJ2MS4xNFwiLCBcInYxLjE1XCJdO1xuICAgICAgfSxcbiAgICB9KTtcblxuICAgIC8vIFByb3BlcnRpZXNcbiAgICBjLnByb3BlcnRpZXMuYWRkT2JqZWN0KHtcbiAgICAgIG5hbWU6IFwia3ViZXJuZXRlc09iamVjdFwiLFxuICAgICAgbGFiZWw6IFwiS3ViZXJuZXRlcyBPYmplY3RcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcE9iamVjdCkge1xuICAgICAgICBwLnJlbGF0aW9uc2hpcHMudXBkYXRlcyh7XG4gICAgICAgICAgcGFydG5lcjoge1xuICAgICAgICAgICAgdHlwZU5hbWU6IFwia3ViZXJuZXRlc0RlcGxveW1lbnRFbnRpdHlcIixcbiAgICAgICAgICAgIG5hbWVzOiBbXCJwcm9wZXJ0aWVzXCIsIFwia3ViZXJuZXRlc09iamVjdFlhbWxcIl0sXG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVsYXRpb25zaGlwcy5laXRoZXIoe1xuICAgICAgICAgIHBhcnRuZXI6IHtcbiAgICAgICAgICAgIHR5cGVOYW1lOiBcImt1YmVybmV0ZXNEZXBsb3ltZW50RW50aXR5XCIsXG4gICAgICAgICAgICBuYW1lczogW1wicHJvcGVydGllc1wiLCBcImt1YmVybmV0ZXNPYmplY3RZYW1sXCJdLFxuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJhcGlWZXJzaW9uXCIsXG4gICAgICAgICAgbGFiZWw6IFwiQVBJIFZlcnNpb25cIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BUZXh0KSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwia2luZFwiLFxuICAgICAgICAgIGxhYmVsOiBcIktpbmRcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BUZXh0KSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAuYmFzZURlZmF1bHRWYWx1ZSA9IFwiRGVwbG95bWVudFwiO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJtZXRhZGF0YVwiLFxuICAgICAgICAgIGxhYmVsOiBcIk1ldGFkYXRhXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBcImt1YmVybmV0ZXNNZXRhZGF0YVwiLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5wcm9wZXJ0aWVzLmFkZE9iamVjdCh7XG4gICAgICAgICAgbmFtZTogXCJzcGVjXCIsXG4gICAgICAgICAgbGFiZWw6IFwiRGVwbG95bWVudCBTcGVjXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wT2JqZWN0KSB7XG4gICAgICAgICAgICBwLnByb3BlcnRpZXMuYWRkTnVtYmVyKHtcbiAgICAgICAgICAgICAgbmFtZTogXCJyZXBsaWNhc1wiLFxuICAgICAgICAgICAgICBsYWJlbDogXCJSZXBsaWNhc1wiLFxuICAgICAgICAgICAgICBvcHRpb25zKHA6IFByb3BOdW1iZXIpIHtcbiAgICAgICAgICAgICAgICBwLm51bWJlcktpbmQgPSBcImludDMyXCI7XG4gICAgICAgICAgICAgIH0sXG4gICAgICAgICAgICB9KTtcbiAgICAgICAgICAgIHAucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICAgICAgbmFtZTogXCJzZWxlY3RvclwiLFxuICAgICAgICAgICAgICBsYWJlbDogXCJTZWxlY3RvclwiLFxuICAgICAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgICAgICB0eXBlTmFtZTogXCJrdWJlcm5ldGVzU2VsZWN0b3JcIixcbiAgICAgICAgICAgICAgICB9O1xuICAgICAgICAgICAgICB9LFxuICAgICAgICAgICAgfSk7XG4gICAgICAgICAgICBwLnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgICAgIG5hbWU6IFwidGVtcGxhdGVcIixcbiAgICAgICAgICAgICAgbGFiZWw6IFwiUG9kIFRlbXBsYXRlIFNwZWNcIixcbiAgICAgICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICAgICAgdHlwZU5hbWU6IFwia3ViZXJuZXRlc1BvZFRlbXBsYXRlU3BlY1wiLFxuICAgICAgICAgICAgICAgIH07XG4gICAgICAgICAgICAgIH0sXG4gICAgICAgICAgICB9KTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICBjLnByb3BlcnRpZXMuYWRkQ29kZSh7XG4gICAgICBuYW1lOiBcImt1YmVybmV0ZXNPYmplY3RZYW1sXCIsXG4gICAgICBsYWJlbDogXCJLdWJlcm5ldGVzIE9iamVjdCBZQU1MXCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BDb2RlKSB7XG4gICAgICAgIHAucmVsYXRpb25zaGlwcy51cGRhdGVzKHtcbiAgICAgICAgICBwYXJ0bmVyOiB7XG4gICAgICAgICAgICB0eXBlTmFtZTogXCJrdWJlcm5ldGVzRGVwbG95bWVudEVudGl0eVwiLFxuICAgICAgICAgICAgbmFtZXM6IFtcInByb3BlcnRpZXNcIiwgXCJrdWJlcm5ldGVzT2JqZWN0XCJdLFxuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlbGF0aW9uc2hpcHMuZWl0aGVyKHtcbiAgICAgICAgICBwYXJ0bmVyOiB7XG4gICAgICAgICAgICB0eXBlTmFtZTogXCJrdWJlcm5ldGVzRGVwbG95bWVudEVudGl0eVwiLFxuICAgICAgICAgICAgbmFtZXM6IFtcInByb3BlcnRpZXNcIiwgXCJrdWJlcm5ldGVzT2JqZWN0XCJdLFxuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLmxhbmd1YWdlID0gXCJ5YW1sXCI7XG4gICAgICB9LFxuICAgIH0pO1xuICB9LFxufSk7Il19