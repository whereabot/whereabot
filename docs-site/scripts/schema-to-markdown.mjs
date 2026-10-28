// Renders JSON Schema objects (draft 2020-12, as schemars writes them) as markdown.

const refName = (ref) => ref.split('/').pop();

/** Follows a `$ref`, merging the referenced object with the schema that points at it. */
export function resolve(schema, defs) {
  if (!schema.$ref) return schema;
  const target = defs[refName(schema.$ref)];
  const { $ref, ...own } = schema;
  return {
    ...target,
    ...own,
    properties: { ...target.properties, ...own.properties },
    required: [...(target.required ?? []), ...(own.required ?? [])],
  };
}

function typeText(schema, defs) {
  const found = resolve(schema, defs);
  if (found.enum) return found.enum.map((value) => `\`${value}\``).join(', ');
  const types = [].concat(found.type ?? []).filter((type) => type !== 'null');
  return `${types.join(' or ') || 'any'}${found.format ? ` (${found.format})` : ''}`;
}

function noteText(schema, defs) {
  const found = resolve(schema, defs);
  const range = found.minimum !== undefined && found.maximum !== undefined ? ` Range ${found.minimum} to ${found.maximum}.` : '';
  return `${found.description ?? schema.description ?? ''}${range}`.trim();
}

/** One row per property. Skips `skip` names. Optional means nullable or not in `required`. */
export function propertyTable(schema, defs, skip = []) {
  const found = resolve(schema, defs);
  const rows = Object.entries(found.properties ?? {})
    .filter(([name]) => !skip.includes(name))
    .map(([name, property]) => {
      const nullable = [].concat(resolve(property, defs).type ?? []).includes('null');
      const required = (found.required ?? []).includes(name) && !nullable ? 'yes' : 'no';
      return `| \`${name}\` | ${typeText(property, defs)} | ${required} | ${noteText(property, defs)} |`;
    });
  if (!rows.length) return 'No fields.\n';
  return ['| Field | Type | Required | Notes |', '| --- | --- | --- | --- |', ...rows].join('\n') + '\n';
}

export const jsonBlock = (value) => '```json\n' + JSON.stringify(value, null, 2) + '\n```\n';
