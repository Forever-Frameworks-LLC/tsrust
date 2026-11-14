const tsx = require('tree-sitter-typescript/tsx/grammar');

module.exports = grammar(tsx, {
  name: 'tsrust',

  conflicts: ($, original) => [...original, [$.primary_expression, $.match_expression]],

  rules: {
    primary_expression: ($, original) => choice(original, alias('match', $.identifier), $.match_expression),

    match_expression: $ => prec.right(seq(
      'match',
      '(',
      field('value', $.expression),
      ')',
      '{',
      repeat($.match_arm),
      optional(alias($.match_final_arm, $.match_arm)),
      '}',
    )),

    match_arm: $ => prec.right(choice(
      seq(field('pattern', $.match_pattern), '=>', field('body', $.statement_block), optional(',')),
      seq(field('pattern', $.match_pattern), '=>', field('value', $.expression), ','),
    )),

    match_final_arm: $ => seq(field('pattern', $.match_pattern), '=>', field('value', $.expression)),

    match_pattern: $ => choice(
      alias('_', $.wildcard_pattern),
      $.string,
      $.number,
      $.true,
      $.false,
      $.null,
      $.undefined,
      seq('-', $.number),
      seq(field('variant', $.identifier), optional(seq('(', field('binding', $.identifier), ')'))),
    ),
  },
});
