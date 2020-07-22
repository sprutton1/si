describe("application", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
  });

  it("create", () => {
    cy.visit("/");
    cy.get("[data-cy=application-nav-link]").click();
    cy.location("pathname").should("match", /^\/o\/(.+)\/w\/(.+)\/a$/);
    cy.get("[data-cy=new-application-button]").click();
    cy.get("[data-cy=new-application-form-application-name]").type("alcest");
    cy.get("[data-cy=new-application-form-create-button]").click();
    cy.get("[data-cy=application-card-name]").contains("alcest");
    cy.get("[data-cy=systems-visualization-default]");
    cy.get("[data-cy=change-set-visualization-open-count]").contains("0");
    cy.get("[data-cy=change-set-visualization-closed-count]").contains("1");
    cy.location("pathname").should("match", /^\/o\/(.+)\/w\/(.+)\/a\/(.+)$/);
  });

  it("list", () => {
    cy.visit("/");
    cy.vuex().should(async (store) => {
      await store.dispatch("application/create", { name: "metallica" });
      await store.dispatch("application/create", { name: "moon tooth" });
    });
    cy.get("[data-cy=application-nav-link]").click();
    cy.get("[data-cy=application-card-name]").contains("metallica");
    cy.get("[data-cy=application-card-name]").contains("moon tooth");
  });
});
