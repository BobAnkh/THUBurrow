var keyword = new Array(10);
keyword = ['火锅 关注 21312312s', 'test', '暗', '辛苦',
    '皮卡丘', '星云', 'bug', '大家', 'hello world',
    'hello 前端 section']
var keyword1 = new Array(10);
keyword1 = ['more burrow', 'Hi there', 'hello world', '你好',
'dd', 'dd1', 'Mofukou', 'Mofukou dd', 'hello world Hi there',
    'sadsdasd']
var postid = new Array(10);
postid = ['#1', '#2', '#3', '#20', '#0', '#'];
var burrowid = new Array(10);
burrowid = ['$1', '$2', '$3', '$2a', '$0', '$'];
var tag = new Array(10);
tag = ['#废话bot', '#剑士', '#废话bot', '#废话bot 剑士', '#THUBURROW'
    , '#bot', '#天文','#tag','#burrow','#11q']
describe('Test Searchpage', function () {
    beforeEach(function () {
        cy.visit('https://frontend-dev.thuburrow.com/login')
        cy.get('input[type=username]')
            .type('荒')
        cy.get('input[type=password]')
            .type('853142213zt')
        cy.get('button[type=submit]').click()
        cy.contains('搜索').click()
        cy.wait(1000)
    });
    it('测试搜帖子关键词', function () {
        for (var i = 0; i < keyword.length; i++) {
            cy.visit('https://frontend-dev.thuburrow.com/searchpage')
            cy.get('input[type=text]').clear().type(keyword[i]).should('have.value', keyword[i])
            cy.get('button[type=button]').closest('.ant-input-group-addon').click()
            cy.wait(1000)
            cy.contains('查看帖子').click()
            cy.contains('查看回复').click()
            cy.contains('回复')
        }
    });

    it('测试搜洞关键词', function () {
        cy.visit('https://frontend-dev.thuburrow.com/searchpage')
        cy.contains('搜帖').click()
        cy.contains('搜洞').click()
        for (var i = 0; i < keyword.length; i++) {
            cy.get('input[type=text]').clear().type(keyword1[i]).should('have.value', keyword1[i])
            cy.get('button[type=button]').closest('.ant-input-group-addon').click()
            cy.wait(2000)
        }
    });

        it('测试搜帖id', function () {
            for (var i = 0; i < postid.length; i++) {
            cy.visit('https://frontend-dev.thuburrow.com/searchpage')
            cy.get('input[type=text]').clear().type(postid[i]).should('have.value', postid[i])
            cy.get('button[type=button]').closest('.ant-input-group-addon').click()
            cy.wait(2000)
            if(i<3)cy.url().should('include',postid[i].replace('#',''))
        }
    });

    it('测试搜洞id', function () {
        for (var i = 0; i < burrowid.length; i++) {
        cy.visit('https://frontend-dev.thuburrow.com/searchpage')
        cy.get('input[type=text]').clear().type(burrowid[i]).should('have.value', burrowid[i])
        cy.get('button[type=button]').closest('.ant-input-group-addon').click()
        cy.wait(2000)
        if(i<3)cy.url().should('include',burrowid[i].replace('$',''))
    }
});
    it('测试搜tag', function () {
        for (var i = 0; i < keyword.length; i++) {
            cy.visit('https://frontend-dev.thuburrow.com/searchpage')
            cy.get('input[type=text]').clear().type(tag[i]).should('have.value', tag[i])
            cy.get('button[type=button]').closest('.ant-input-group-addon').click()
            cy.wait(2000)
        }
    });
  });
