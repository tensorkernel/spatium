// Spatium Stripe webhook handler (STUB).
//
// Per user direction (Session 4): "we will do payment integration, signing, part
// later". This file is a stub that:
//
// 1. Listens on http://localhost:8082/stripe/webhook in dev.
// 2. Accepts a Stripe webhook event and logs it (does NOT verify the signature
//    yet — that requires a Stripe webhook secret in production).
// 3. Returns 200 OK so Stripe doesn't retry.
//
// Phase 6 production will:
// - Verify the Stripe-Signature header with `STRIPE_WEBHOOK_SECRET`.
// - Handle `checkout.session.completed` → issue a license key, send email.
// - Handle `customer.subscription.deleted` / `unpaid` → revoke license.
// - Handle `charge.refunded` → revoke on full refund.
// - Handle `invoice.payment_succeeded` → extend `expires_at` for yearly renewals.

import cors from '@fastify/cors';
import Fastify from 'fastify';

const PORT = Number(process.env.STRIPE_WEBHOOK_PORT ?? 8082);
const HOST = process.env.STRIPE_WEBHOOK_HOST ?? '127.0.0.1';

const server = Fastify({ logger: { level: 'info' } });
await server.register(cors, { origin: true });

server.post('/stripe/webhook', async (req, reply) => {
  const sig = req.headers['stripe-signature'];
  const event = req.body as { type?: string; id?: string; data?: unknown };

  // Phase 6 production: verify the signature with the Stripe webhook secret.
  // For now (stub), just log.
  server.log.info(
    { sig, eventType: event?.type, eventId: event?.id },
    'Stripe webhook received (stub)',
  );

  // Per the event type, dispatch (stub).
  switch (event?.type) {
    case 'checkout.session.completed':
      // TODO Phase 6: issue a license key, send email with activation link.
      server.log.info('checkout.session.completed — would issue license key');
      break;
    case 'customer.subscription.deleted':
    case 'customer.subscription.unpaid':
      // TODO Phase 6: revoke the license.
      server.log.info(`${event.type} — would revoke license`);
      break;
    case 'charge.refunded':
      // TODO Phase 6: revoke on full refund.
      server.log.info('charge.refunded — would revoke license on full refund');
      break;
    case 'invoice.payment_succeeded':
      // TODO Phase 6: extend expires_at for yearly renewals.
      server.log.info('invoice.payment_succeeded — would extend yearly renewal');
      break;
    default:
      server.log.info({ type: event?.type }, 'unhandled event type (stub)');
  }

  // Stripe expects 200 OK to stop retrying.
  return reply.send({ received: true });
});

server.get('/health', async () => ({ status: 'ok', service: 'stripe-webhook-stub' }));

try {
  await server.listen({ port: PORT, host: HOST });
  console.log(`[stripe-webhook] Stub listening on http://${HOST}:${PORT}/stripe/webhook`);
  console.log(
    `[stripe-webhook] Phase 6 production: verify signatures with STRIPE_WEBHOOK_SECRET, then issue/revoke licenses`,
  );
} catch (err) {
  console.error(err);
  process.exit(1);
}
