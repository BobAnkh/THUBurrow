// holepage.spec.js created with Cypress
//
// Start writing your Cypress tests below!
// If you're unfamiliar with how Cypress works,
// check out the link below and learn how to write your first test:
// https://on.cypress.io/writing-first-test
// <reference types="cypress" />

// const { AsyncLocalStorage } = require("async_hooks")

// Welcome to Cypress!
//
// This spec file contains a variety of sample tests
// for a todo list app that are designed to demonstrate
// the power of writing tests in Cypress.
//
// To learn more about how Cypress works and
// what makes it such an awesome testing tool,
// please read our getting started guide:
// https://on.cypress.io/introduction-to-cypress
var useCase = new Array(5);
useCase[0] = {
  Username: 'lcf0428',
  Password: 'wcsWFS321',
  PasswordConfirm: 'wcsWFS321',
  Email: 'liucf18',
  EmailConfirm: '214325',
};
useCase[1] = {
  Username: 'lcf0428',
  Password: 'WCSFAZADa',
  PasswordConfirm: 'WCSFAZKs',
  Email: 'liucf18',
  EmailConfirm: '214325',
};
useCase[2] = {
  Username: 'lcf0428',
  Password: 'wcsWFS321',
  PasswordConfirm: 'wcsWFS321',
  Email: 'liucf18&',
  EmailConfirm: '214325',
};
useCase[3] = {
  Username: '   ',
  Password: 'wcsWFS321',
  PasswordConfirm: 'wcsWFS321',
  Email: 'liucf18',
  EmailConfirm: '214325',
};
useCase[4] = {
  Username: 'lcf0428',
  Password: 'wcsWF',
  PasswordConfirm: 'wcsWF',
  Email: 'liucf18',
  EmailConfirm: '     ',
};

describe('burrow page', () => {
  beforeEach(() => {
    // Cypress starts out with a blank slate for each test
    // so we must tell it to visit our website with the `cy.visit()` command.
    // Since we want to visit the same URL at the start of all our tests,
    // we include it in our beforeEach function so that it runs before each test
    // cy.visit('http://localhost:3000/login')
  });

  it('clink the button "post"', function () {
    for (var i = 0; i < useCase.length; i++) {
      cy.visit('http://localhost:3000/login');
      cy.contains('注册')
        .click()
        .get('input[name=Username]')
        .type(useCase[i].Username)
        .should('have.value', useCase[i].Username);
      cy.get('input[name=Password]')
        .type(useCase[i].Password)
        .should('have.value', useCase[i].Password);
      cy.get('input[name=Password_Confirm]')
        .type(useCase[i].PasswordConfirm)
        .should('have.value', useCase[i].PasswordConfirm);
      cy.get('input[name=Email]')
        .type(useCase[i].Email)
        .should('have.value', useCase[i].Email);
      cy.get('input[name=Email_Confirm]')
        .type(useCase[i].EmailConfirm)
        .should('have.value', useCase[i].EmailConfirm);
      cy.get('input[id=register_agreement]').click();
      cy.get('button[type=submit]').click();
      if (i === 0) {
        cy.contains('即刻');
      } else {
        cy.wait(6000);
        cy.contains('服务条款');
      }
    }
  });
});
