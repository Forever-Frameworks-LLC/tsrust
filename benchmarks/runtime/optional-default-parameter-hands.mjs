// Literal expectations, not calls to the scenario oracle. No execution acceptance is claimed yet.
export const hands = [
 ['optional_parameter_int_guard',1,0,'x',[1000,1]], ['optional_parameter_int_guard',1,2,'x',[5,1]],
 ['default_parameter_number_value',1,0,'x',[[25,1,1]]], ['default_parameter_number_value',1,2,'x',[[5e-324,1,0]]],
 ['default_parameter_boolean_value',1,0,'x',[true,false,false,false,true]], ['default_parameter_boolean_value',1,3,'x',[false,false,true,true,false]],
 ['default_parameter_string_value',1,0,'é',['é:default:0','1']], ['default_parameter_string_value',1,2,'é',['ééé:present:2','0']],
 ['default_parameter_array_value',1,0,'x',[[0,1],[1,1]]], ['default_parameter_array_value',1,2,'x',[[2,3,4,5],[1,0]]],
 ['default_parameter_record_value',1,0,'é',[['record','100','é:default'],['1','1']]], ['default_parameter_record_value',1,3,'é',[['null'],['1','0']]],
 ['default_parameter_dependent_positions',1,0,'x',[[10,0,11,21],[1,2]]], ['default_parameter_dependent_positions',1,3,'x',[[13,3,23,36],[1,1]]],
 ['optional_parameter_read_callback',1,0,'x',[[],[0,0]]], ['optional_parameter_read_callback',1,2,'x',[[7],[1,0]]],
 ['optional_parameter_mutate_callback',1,0,'x',[[],[0,0]]], ['optional_parameter_mutate_callback',1,2,'x',[[5],[1,0]]],
 ['default_parameter_owned_reusable_callback',1,0,'x',[[101,103],[2,1]]], ['default_parameter_owned_reusable_callback',1,2,'x',[[5,7],[2,0]]],
 ['optional_parameter_once_callback',1,0,'é',[[],['0','0']]], ['optional_parameter_once_callback',1,2,'é',[['ééé:present:2'],['1','0']]],
 ['default_parameter_owned_once_callback',1,0,'é',[['é:default:0'],['1','1']]], ['default_parameter_owned_once_callback',1,2,'é',[['ééé:present:2'],['1','0']]],
 ['default_parameter_stored_family',1,0,'x',[[11],[1,1]]], ['default_parameter_stored_family',1,1,'x',[[102],[1,1]]],
 ['default_parameter_number_value',1,5,'x',[[Infinity,1,0]]], ['default_parameter_number_value',1,11,'x',[[-2.5,1,0]]],
 ['default_parameter_number_value',1,17,'x',[[-0,1,0]]], ['default_parameter_number_value',1,23,'x',[[NaN,1,0]]],
 ['default_parameter_number_value',1,8,'x',[[0,1,0]]], ['default_parameter_number_value',1,14,'x',[[-Infinity,1,0]]], ['default_parameter_number_value',1,20,'x',[[.5,1,0]]],
 ['default_parameter_number_value',0,0,'x',[]], ['default_parameter_array_value',0,0,'x',[]]
];
