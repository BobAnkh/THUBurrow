describe('Test Profile page', function () {
  beforeEach(function () {
    cy.visit('/profile');
  });
  it('测试获取到数据', function () {
    cy.contains('我的地洞').parents('.ant-card.ant-card-bordered').find('.ant-table-tbody > tr').should('have.length.greaterThan', 0)
    cy.contains('关注的洞').parents('.ant-card.ant-card-bordered').find('.ant-table-tbody > tr').should('have.length.greaterThan', 0)
    cy.contains('收藏的帖子').parents('.ant-card.ant-card-bordered').find('.ant-list-items > li').should('have.length.greaterThan', 0)
  });
  it('测试我的地洞跳转到对应的洞', function(){
    cy.contains('我的地洞').parents('.ant-card.ant-card-bordered').find('.ant-table-tbody > tr:first-child').as('burrow');
    cy.waitUntil(() => cy.get('@burrow').find('.ant-table-cell').first().invoke('text').then((text) => {text !== ''}));
    cy.get('@burrow').find('td').first().invoke('text').then((text) => {
      const burrow_id = text;
      cy.log(burrow_id);
      cy.wait(2000);
      cy.get('@burrow').find('td').eq(1).click();
      cy.url().should('include', `/burrow/${burrow_id}`)
    });
  })
  it('测试跳转到对应的帖子', function(){
    cy.contains('收藏的帖子').parents('.ant-card.ant-card-bordered').find('.ant-list-items > li:first-child').contains('#').then(function ($a) {
      const post_id = $a.text().split(' ')[0].replace('#','');
      cy.wrap($a).click();
      cy.url().should('include', `/post/${post_id}`)
    })
  })

  it('测试编辑我的地洞', function(){
    cy.contains('我的地洞').parents('.ant-card.ant-card-bordered').find('.ant-table-tbody > tr:first-child').as('burrow');
    cy.waitUntil(() => cy.get('@burrow').find('.ant-table-cell').first().invoke('text').then((text) => {text !== ''}));
    cy.get('@burrow').contains('编辑').click();
    // cy.contains('我的地洞').parents('.ant-card.ant-card-bordered').find('.ant-table-tbody > tr:first-child').as('burrow');
    cy.get('@burrow').find('.editable-cell-value-wrap').first().dblclick();
    const test_input = 'cypress test message'
    cy.get('@burrow').find('input').clear().type(`${test_input}{enter}`);
    cy.get('@burrow').find('.editable-cell-value-wrap').first().contains(test_input);
    cy.get('@burrow').find('.editable-cell-value-wrap').eq(1).dblclick();
    cy.get('@burrow').find('input').clear().type(`${test_input}{enter}`);
    cy.get('@burrow').find('.editable-cell-value-wrap').eq(1).contains(test_input);
    cy.get('@burrow').contains('保存').click();
    cy.get('@burrow').find('td').eq(1).contains(test_input); //find('td').eq(1)表示第2个满足选择器'td'的对象
    cy.get('@burrow').find('td').eq(2).contains(test_input);
  })

  it('测试新增地洞', function(){
    cy.contains('我的地洞').parents('.ant-card.ant-card-bordered').find('.ant-table-tbody > tr:last-child').as('burrow');
    cy.waitUntil(() => cy.get('@burrow').find('.ant-table-cell').first().invoke('text').then((text) => {text !== ''}));
    cy.contains('+ 新增地洞').click();
    cy.contains('我的地洞').parents('.ant-card.ant-card-bordered').find('.ant-table-tbody > tr:last-child').as('burrow');
    const test_input = 'cypress test message'
    cy.get('@burrow').find('input').eq(0).type(`${test_input}`);
    cy.get('@burrow').find('input').eq(1).type(`${test_input}{enter}`);
    cy.get('@burrow').find('.editable-cell-value-wrap').eq(0).contains(test_input);
    cy.get('@burrow').contains('保存').click();
    cy.get('@burrow').find('td').eq(1).contains(test_input);
    cy.get('@burrow').find('td').eq(2).contains(test_input);
    cy.get('@burrow').find('td').eq(0).invoke('text').then((text) => expect(text).to.not.equal('').and.not.equal('待分配')) // 正确分配了新的洞号
  })
});
