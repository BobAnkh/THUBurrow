
var useCase = new Array(6);
useCase[0] = { Username: 'test', Password: 'zt853142213', tag: true };
useCase[1] = { Username: 'test', Password: '1231', tag: false };
useCase[1] = {Username: 'test', Password: '11111111111111111111111111111111fsdfsd231',tag:false};
useCase[5] = { Username: '  `', Password: 'WCSFAZADa', tag:false};
useCase[2] = {Username: '1', Password: '   ',tag:false};
useCase[3] = {Username: '-q', Password: '123456q',tag:false};
useCase[4] = {Username: '  ', Password: 'wcsWF', tag: false };
 useCase[6] = {Username: '荒', Password: '853142213zt',tag:true};
describe('login test', () => {
    it('clink the button "post"', function () {
    for (var i = 0; i < useCase.length; i++) {
            cy.visit('https://frontend-dev.thuburrow.com/login')
            cy.get('input[type=username]')
                .type(useCase[i].Username).should('have.value', useCase[i].Username)
            cy.get('input[type=password]')
                .type(useCase[i].Password).should('have.value', useCase[i].Password)
            cy.get('button[type=submit]').click()
            if (useCase[i].tag == true) {
                cy.url().should('include', 'home')     //验证目标url 是否正确包含关键字
                cy.title().should('contain', '首页')  //验证页面 title 是否正确
            }
            else {
                cy.wait(1000)
                cy.contains('忘记账号')
            }
            cy.screenshot()  //截屏
        }
    })
  })
  