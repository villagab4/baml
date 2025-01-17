// This file is auto-generated. Do not edit this file manually.
//
// Disable formatting for this file to avoid linting errors.
// tslint:disable
// @ts-nocheck

import { Deserializer } from '../../baml_lib/deserializer/deserializer';
import { Main } from '../client';
import { ExtractMeetingRequestInfo } from '../function';
import { schema } from '../json_schema';
import { InternalConversation } from '../types_internal';


const prompt_template = `\
Given a user is trying to schedule a meeting, extract the relevant
information from the query.

Context:
\`\`\`
Today is {//BAML_CLIENT_REPLACE_ME_MAGIC_input.now//}
\`\`\`

Conversation:
\`\`\`
{//BAML_CLIENT_REPLACE_ME_MAGIC_input.convo.display//}
\`\`\`

Output JSON:
{
  "time": string | null,
  "duration": string | null,
  "attendees": string[],
  "topic": string | null
}

JSON:\
`;

const deserializer = new Deserializer<MeetingRequest>(schema, {
  $ref: '#/definitions/ExtractMeetingRequestInfo_output'
});

ExtractMeetingRequestInfo.registerImpl('robust', {
  name: 'robust',
  run: async (
args: {
  convo: Conversation, now: string
}
  ): Promise<MeetingRequest> => {
  const convo = InternalConversation.from(args.convo);
  const now = args.now;
  
    const result = await Main.run_prompt_template(
      prompt_template,
      {
        "{//BAML_CLIENT_REPLACE_ME_MAGIC_input.now//}": now,
        "{//BAML_CLIENT_REPLACE_ME_MAGIC_input.convo.display//}": convo.display,
      }
    );

    return deserializer.coerce(result);
  }
});
