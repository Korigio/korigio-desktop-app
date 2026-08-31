type Props = {
  text: string;
};

export function PrintAcknowledgment({ text }: Props) {
  return (
    <section className="mt-8">
      <p className="text-sm leading-relaxed">{text}</p>
    </section>
  );
}
