type Props = {
  text: string;
};

export function PrintAcknowledgment({ text }: Props) {
  return (
    <section className="mt-6">
      <p className="text-sm leading-relaxed">{text}</p>
    </section>
  );
}
