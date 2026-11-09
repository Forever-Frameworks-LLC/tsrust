import { View, Text, Node, signal, run, Questionnaire, Question, QuestionChoice } from 'std:ui';

function App(): Node {
  const answers = signal('none');
  return (
    <View className="flex-col size-full p-6 gap-4">
      <Questionnaire onSubmit={(result) => { answers.set(clone(result)); }}>
        <Question id="plan" title="Which plan fits your team?" description="You can change it later." required={true}>
          <QuestionChoice id="free" label="Free" description="For individuals." />
          <QuestionChoice id="pro" label="Pro" />
        </Question>
        <Question id="tools" title="Which tools do you use?" multiple={true}>
          <QuestionChoice id="editor" label="Editor" />
          <QuestionChoice id="terminal" label="Terminal" />
        </Question>
      </Questionnaire>
      <Text>Answers {answers}</Text>
    </View>
  );
}

function main() {
  unwrap(run(App(), 'Questionnaire', {
    width: 720, height: 560
  }));
}
