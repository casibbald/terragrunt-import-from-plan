package test

import (
	"testing"
	"strings"

	"github.com/gruntwork-io/terratest/modules/terraform"
	"github.com/stretchr/testify/assert"
)

func TestKmsModule(t *testing.T) {
	terraformOptions := terraform.WithDefaultRetryableErrors(t, &terraform.Options{
		TerraformDir: "../",

		Vars: map[string]interface{}{
			"project_id": "test-project",
			"key_ring_name": "test-key-ring",
			"location": "us-central1",
		},
	})

	defer terraform.Destroy(t, terraformOptions)

	terraform.InitAndApply(t, terraformOptions)

	cryptoKeyName := terraform.Output(t, terraformOptions, "crypto_key_name")

	assert.NotEmpty(t, cryptoKeyName)
	assert.True(t, strings.Contains(cryptoKeyName, "example"),
		"Expected crypto key name to contain 'example'")
}